use clap::Parser;
use fedimint_core::{anyhow, util::SafeUrl};
use fedimint_eventlog::EventLogId;
use fedimint_logging::TracingSetup;
use incoming::{
    CompleteLightningPaymentSucceeded, LNv1IncomingPaymentFailed, LNv1IncomingPaymentStarted,
    LNv1IncomingPaymentSucceeded,
};
use ln_gateway::rpc::{rpc_client::GatewayRpcClient, PaymentLogPayload};
use outgoing::{
    LNv1OutgoingPaymentFailed, LNv1OutgoingPaymentStarted, LNv1OutgoingPaymentSucceeded,
};
use tokio_postgres::{Client, NoTls};
use tracing::{error, info, warn};

mod incoming;
mod outgoing;

#[derive(Parser, Debug)]
struct GatewayETLOpts {
    /// Gateway HTTP Address
    #[arg(long = "gateway-addr", env = "GATEWAY_ADDRESS")]
    gateway_addr: SafeUrl,

    /// Gateway Password
    #[arg(long = "password", env = "GATEWAY_PASSWORD")]
    password: String,

    /// Telegram Bot token
    #[arg(long = "bot-token", env = "BOT_TOKEN")]
    bot_token: String,

    /// Telegram Chat ID
    #[arg(long = "chat-id", env = "CHAT_ID")]
    chat_id: String,

    #[arg(long = "db-host", env = "DB_HOST")]
    db_host: String,

    #[arg(long = "db-user", env = "DB_USER")]
    db_user: String,

    #[arg(long = "db-password", env = "DB_PASSWORD")]
    db_password: String,

    #[arg(long = "db-name", env = "DB_NAME")]
    db_name: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    TracingSetup::default().init()?;
    let opts = GatewayETLOpts::parse();
    let client = GatewayRpcClient::new(opts.gateway_addr, Some(opts.password));

    let pg_client = connect_pg(opts.db_host, opts.db_user, opts.db_password, opts.db_name).await?;

    // This needs to be per federation
    let max_log_id = get_max_log_id(&pg_client).await?;

    info!(?max_log_id, "Getting gateway info...");
    let info = client.get_info().await?;
    for fed_info in info.federations {
        {
            let federation_id = fed_info.federation_id;
            let federation_name = fed_info.federation_name.clone().expect("should be present");

            let payment_log = client
                .payment_log(PaymentLogPayload {
                    end_position: None,
                    pagination_size: usize::MAX,
                    federation_id,
                    event_kinds: vec![],
                })
                .await?;

            let mut outgoing_payment_started = 0;
            let mut outgoing_payment_succeeded = 0;
            let mut outgoing_payment_failed = 0;
            let mut incoming_payment_started = 0;
            let mut incoming_payment_succeeded = 0;
            let mut incoming_payment_failed = 0;
            let mut complete_lightning_payment_succeeded = 0;
            for (log_id, kind, module, timestamp, value) in payment_log.0 {
                if parse_log_id(&log_id) <= max_log_id {
                    break;
                }

                let kind = parse_event_kind(format!("{kind:?}"));
                let module = module.expect("Should be present").0.as_str().to_string();
                match (kind.as_str(), module.as_str()) {
                    ("outgoing-payment-started", "ln") => {
                        let outgoing_payment_started_event: LNv1OutgoingPaymentStarted =
                            serde_json::from_value(value).expect("Could not parse event");
                        outgoing_payment_started_event
                            .insert(
                                &pg_client,
                                &log_id,
                                timestamp,
                                &federation_id,
                                federation_name.clone(),
                            )
                            .await?;
                        outgoing_payment_started += 1;
                    }
                    ("outgoing-payment-succeeded", "ln") => {
                        let outgoing_payment_succeeded_event: LNv1OutgoingPaymentSucceeded =
                            serde_json::from_value(value).expect("Could not parse event");
                        outgoing_payment_succeeded_event
                            .insert(
                                &pg_client,
                                &log_id,
                                timestamp,
                                &federation_id,
                                federation_name.clone(),
                            )
                            .await?;
                        outgoing_payment_succeeded += 1;
                    }
                    ("outgoing-payment-failed", "ln") => {
                        let outgoing_payment_failed_event: LNv1OutgoingPaymentFailed =
                            serde_json::from_value(value).expect("Could not parse event");
                        outgoing_payment_failed_event
                            .insert(
                                &pg_client,
                                &log_id,
                                timestamp,
                                &federation_id,
                                federation_name.clone(),
                            )
                            .await?;
                        outgoing_payment_failed += 1;
                    }
                    ("incoming-payment-started", "ln") => {
                        let incoming_payment_started_event: LNv1IncomingPaymentStarted =
                            serde_json::from_value(value).expect("Could not parse event");
                        incoming_payment_started_event
                            .insert(
                                &pg_client,
                                &log_id,
                                timestamp,
                                &federation_id,
                                federation_name.clone(),
                            )
                            .await?;
                        incoming_payment_started += 1;
                    }
                    ("incoming-payment-succeeded", "ln") => {
                        let incoming_payment_succeeded_event: LNv1IncomingPaymentSucceeded =
                            serde_json::from_value(value).expect("Could not parse event");
                        incoming_payment_succeeded_event
                            .insert(
                                &pg_client,
                                &log_id,
                                timestamp,
                                &federation_id,
                                federation_name.clone(),
                            )
                            .await?;
                        incoming_payment_succeeded += 1;
                    }
                    ("incoming-payment-failed", "ln") => {
                        let incoming_payment_failed_event: LNv1IncomingPaymentFailed =
                            serde_json::from_value(value).expect("Could not parse event");
                        incoming_payment_failed_event
                            .insert(
                                &pg_client,
                                &log_id,
                                timestamp,
                                &federation_id,
                                federation_name.clone(),
                            )
                            .await?;
                        incoming_payment_failed += 1;
                    }
                    ("complete-lightning-payment-succeeded", "ln") => {
                        let complete_lightning_payment_succeeded_event: CompleteLightningPaymentSucceeded = serde_json::from_value(value).expect("Could not parse event");
                        complete_lightning_payment_succeeded_event
                            .insert(
                                &pg_client,
                                &log_id,
                                timestamp,
                                &federation_id,
                                federation_name.clone(),
                            )
                            .await?;
                        complete_lightning_payment_succeeded += 1;
                    }
                    (kind, module) => {
                        warn!(?kind, ?module, "Unknown event kind or module");
                    }
                }
            }

            let total_events = outgoing_payment_started
                + outgoing_payment_succeeded
                + outgoing_payment_failed
                + incoming_payment_started
                + incoming_payment_succeeded
                + incoming_payment_failed
                + complete_lightning_payment_succeeded;
            info!(?total_events, ?outgoing_payment_started, ?outgoing_payment_succeeded, ?outgoing_payment_failed, ?incoming_payment_started, ?incoming_payment_succeeded, ?incoming_payment_failed, ?complete_lightning_payment_succeeded, federation_name = %fed_info.federation_name.clone().expect("Name should be present"), "Total Events");
        }
    }

    Ok(())
}

// TODO: add federation ID to where clause
async fn get_max_log_id(pg_client: &Client) -> anyhow::Result<i64> {
    let query = "
        SELECT MAX(log_id)
        FROM (
            SELECT log_id FROM lnv1_outgoing_payment_started 
            UNION ALL
            SELECT log_id FROM lnv1_outgoing_payment_succeeded
            UNION ALL
            SELECT log_id FROM lnv1_outgoing_payment_failed
            UNION ALL
            SELECT log_id FROM lnv1_incoming_payment_started
            UNION ALL
            SELECT log_id FROM lnv1_incoming_payment_succeeded
            UNION ALL
            SELECT log_id FROM lnv1_incoming_payment_failed
            UNION ALL
            SELECT log_id FROM lnv1_complete_lightning_payment_succeeded
        ) AS combined_log_ids
    ";

    let rows = pg_client.query(query, &[]).await?;
    if let Some(row) = rows.get(0) {
        let max_log_id: Option<i64> = row.get(0);
        if let Some(max_log_id) = max_log_id {
            return Ok(max_log_id);
        }
    }

    Ok(0)
}

async fn connect_pg(
    db_host: String,
    db_user: String,
    db_password: String,
    db_name: String,
) -> anyhow::Result<Client> {
    info!("Connecting to database...");
    let (pg_client, pg_connection) = tokio_postgres::connect(
        format!("host={db_host} user={db_user} password={db_password} dbname={db_name}").as_str(),
        NoTls,
    )
    .await?;

    tokio::spawn(async move {
        if let Err(err) = pg_connection.await {
            error!(?err, "Postgres connection error");
        }
    });

    Ok(pg_client)
}

// TODO: Remove this once EventKind can be parsed correctly
fn parse_event_kind(input: String) -> String {
    if let Some(start) = input.find('(') {
        if let Some(end) = input.rfind(')') {
            let extracted = &input[start + 2..end - 1]; // Skip `("` and `")`
            return extracted.to_string();
        }
    }

    panic!("Malformatted String");
}

// TODO: Remove this once LogId can be used as a u64
pub fn parse_log_id(log_id: &EventLogId) -> i64 {
    let input = format!("{log_id:?}");
    if let Some(start) = input.find('(') {
        if let Some(end) = input.find(')') {
            let number_str = &input[start + 1..end]; // Extract substring inside parentheses
            if let Ok(number) = number_str.parse::<i64>() {
                return number;
            }
        }
    }

    panic!("Malformatted event log id");
}
