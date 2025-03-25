use chrono::DateTime;
use clap::Parser;
use fedimint_core::{anyhow, config::FederationId, util::SafeUrl};
use fedimint_eventlog::EventLogId;
use fedimint_logging::TracingSetup;
use ln_gateway::rpc::{rpc_client::GatewayRpcClient, PaymentLogPayload};
use outgoing::Lnv1OutgoingPaymentStarted;
use serde_json::Value;
use tokio_postgres::{Client, NoTls};
use tracing::{error, info, warn};

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
                match kind.as_str() {
                    "outgoing-payment-started" => {
                        let outgoing_payment_started_event: Lnv1OutgoingPaymentStarted = serde_json::from_value(value).expect("Could not parse LNv1OutgoingPaymentStarted");
                        outgoing_payment_started_event.insert(&pg_client, &log_id, timestamp, &federation_id, federation_name.clone()).await?;
                        outgoing_payment_started += 1;
                    }
                    "outgoing-payment-succeeded" => {
                        let contract_id = value["contract_id"]
                            .as_str()
                            .expect("Should be present")
                            .to_string();
                        let contract_amount = value["outgoing_contract"]["amount"].as_u64().expect("contract amount should be present");
                        let gateway_key = value["outgoing_contract"]["contract"]["gateway_key"].as_str().expect("Should be present").to_string();
                        let payment_hash = value["outgoing_contract"]["contract"]["hash"].as_str().expect("Should be present").to_string();
                        let timelock = value["outgoing_contract"]["contract"]["timelock"].as_u64().expect("Should be present");
                        let user_key = value["outgoing_contract"]["contract"]["user_key"].as_str().expect("Should be present").to_string();
                        let preimage = value["preimage"].as_str().expect("Should be present").to_string();
                        insert_outgoing_payment_succeeded(&pg_client, &log_id, timestamp, federation_id, federation_name.clone(), contract_id, contract_amount as i64, gateway_key, payment_hash, timelock as i64, user_key, preimage).await?;
                        outgoing_payment_succeeded += 1;
                    }
                    "outgoing-payment-failed" => {
                        let contract_id = value["contract_id"]
                            .as_str()
                            .expect("Should be present")
                            .to_string();
                        let contract_amount = value["outgoing_contract"]["amount"].as_u64().expect("contract amount should be present");
                        let gateway_key = value["outgoing_contract"]["contract"]["gateway_key"].as_str().expect("Should be present").to_string();
                        let payment_hash = value["outgoing_contract"]["contract"]["hash"].as_str().expect("Should be present").to_string();
                        let timelock = value["outgoing_contract"]["contract"]["timelock"].as_u64().expect("Should be present");
                        let user_key = value["outgoing_contract"]["contract"]["user_key"].as_str().expect("Should be present").to_string();
                        let error_reason = extract_error_reason(value)?;
                        insert_outgoing_payment_failed(&pg_client, &log_id, timestamp, federation_id, federation_name.clone(), contract_id, contract_amount as i64, gateway_key, payment_hash, timelock as i64, user_key, error_reason).await?;
                        outgoing_payment_failed += 1;
                    }
                    "incoming-payment-started" => {
                        let contract_id = value["contract_id"]
                            .as_str()
                            .expect("Should be present")
                            .to_string();
                        let contract_amount = value["contract_amount"].as_u64().expect("contract amount should be present");
                        let invoice_amount = value["invoice_amount"].as_u64().expect("invoice amount should be present");
                        let operation_id = value["operation_id"]
                            .as_str()
                            .expect("Should be present")
                            .to_string();
                        let payment_hash = value["payment_hash"]
                            .as_str()
                            .expect("Should be present")
                            .to_string();
                        insert_incoming_payment_started(&pg_client, &log_id, timestamp, federation_id, federation_name.clone(), contract_id, contract_amount as i64, invoice_amount as i64, operation_id, payment_hash).await?;
                        incoming_payment_started += 1;
                    }
                    "incoming-payment-succeeded" => {
                        let payment_hash = value["payment_hash"]
                            .as_str()
                            .expect("Should be present")
                            .to_string();
                        let preimage = value["preimage"]
                            .as_str()
                            .expect("Should be present")
                            .to_string();
                        insert_incoming_payment_succeeded(&pg_client, &log_id, timestamp, federation_id, federation_name.clone(), payment_hash, preimage).await?;
                        incoming_payment_succeeded += 1;
                    }
                    "incoming-payment-failed" => {
                        let payment_hash = value["payment_hash"]
                            .as_str()
                            .expect("Should be present")
                            .to_string();
                        let error = value["error"]
                            .as_str()
                            .expect("Should be present")
                            .to_string();
                        insert_incoming_payment_failed(&pg_client, &log_id,timestamp, federation_id, federation_name.clone(), payment_hash, error).await?;
                        incoming_payment_failed += 1;
                    }
                    "complete-lightning-payment-succeeded" => {
                        let payment_hash = value["payment_hash"]
                            .as_str()
                            .expect("Should be present")
                            .to_string();
                        insert_complete_lightning_payment_succeeded(&pg_client, &log_id, timestamp, federation_id, federation_name.clone(), payment_hash).await?;
                        complete_lightning_payment_succeeded += 1;
                    }
                    kind => {
                        warn!(?kind, "Unknown event kind");
                    }
                }
            }

            let total_events = outgoing_payment_started + outgoing_payment_succeeded + outgoing_payment_failed + incoming_payment_started + incoming_payment_succeeded + incoming_payment_failed + complete_lightning_payment_succeeded;
            info!(?total_events, ?outgoing_payment_started, ?outgoing_payment_succeeded, ?outgoing_payment_failed, ?incoming_payment_started, ?incoming_payment_succeeded, ?incoming_payment_failed, ?complete_lightning_payment_succeeded, federation_name = %fed_info.federation_name.clone().expect("Name should be present"), "Total Events");
        }
    }

    Ok(())
}

fn extract_error_reason(data: Value) -> anyhow::Result<Option<String>> {
    // Check for the 'error_type' key and handle different types of errors
    if let Some(error) = data.get("error") {
        if let Some(error_type) = error.get("error_type") {
            if let Some(lightning_error) = error_type.get("LightningPayError") {
                if let Some(failed_payment) = lightning_error.get("lightning_error") {
                    if let Some(failure_reason) = failed_payment.get("FailedPayment")
                        .and_then(|e| e.get("failure_reason")) {
                        return Ok(Some(failure_reason.as_str().unwrap_or_default().to_string()));
                    }
                }
            } else if let Some(invalid_outgoing_contract) = error_type.get("InvalidOutgoingContract") {
                if let Some(invoice_expired) = invalid_outgoing_contract.get("error")
                    .and_then(|e| e.get("InvoiceExpired")) {
                    return Ok(Some(format!("Invoice expired: {}", invoice_expired.as_i64().unwrap_or_default())));
                }
            }
        }
    }

    // Return None if no error reason is found
    Ok(None)
}

async fn insert_complete_lightning_payment_succeeded(
    pg_client: &Client,
    log_id: &EventLogId,
    ts: u64,
    federation_id: FederationId,
    federation_name: String,
    payment_hash: String,
) -> anyhow::Result<()> {
    let log_id = parse_log_id(&log_id);
    let timestamp = DateTime::from_timestamp_micros(ts as i64)
        .expect("Should convert DateTime correctly")
        .naive_utc();
    pg_client.execute("INSERT INTO lnv1_complete_lightning_payment_succeeded (log_id, ts, federation_id, federation_name, payment_hash) VALUES ($1, $2, $3, $4, $5)",
    &[&log_id, &timestamp, &federation_id.to_string(), &federation_name, &payment_hash]).await?;
    Ok(())
}

async fn insert_incoming_payment_failed(
    pg_client: &Client,
    log_id: &EventLogId,
    ts: u64,
    federation_id: FederationId,
    federation_name: String,
    payment_hash: String,
    error: String,
) -> anyhow::Result<()> {
    let log_id = parse_log_id(&log_id);
    let timestamp = DateTime::from_timestamp_micros(ts as i64)
        .expect("Should convert DateTime correctly")
        .naive_utc();
    pg_client.execute("INSERT INTO lnv1_incoming_payment_failed (log_id, ts, federation_id, federation_name, payment_hash, error_reason) VALUES ($1, $2, $3, $4, $5, $6)",
    &[&log_id, &timestamp, &federation_id.to_string(), &federation_name, &payment_hash, &error]).await?;
    Ok(())
}

async fn insert_incoming_payment_succeeded(
    pg_client: &Client,
    log_id: &EventLogId,
    ts: u64,
    federation_id: FederationId,
    federation_name: String,
    payment_hash: String,
    preimage: String,
) -> anyhow::Result<()> {
    let log_id = parse_log_id(&log_id);
    let timestamp = DateTime::from_timestamp_micros(ts as i64)
        .expect("Should convert DateTime correctly")
        .naive_utc();
    pg_client.execute("INSERT INTO lnv1_incoming_payment_succeeded (log_id, ts, federation_id, federation_name, payment_hash, preimage) VALUES ($1, $2, $3, $4, $5, $6)",
    &[&log_id, &timestamp, &federation_id.to_string(), &federation_name, &payment_hash, &preimage]).await?;
    Ok(())
}

async fn insert_incoming_payment_started(
    pg_client: &Client,
    log_id: &EventLogId,
    ts: u64,
    federation_id: FederationId,
    federation_name: String,
    contract_id: String,
    contract_amount: i64,
    invoice_amount: i64,
    operation_id: String,
    payment_hash: String,
) -> anyhow::Result<()> {
    let log_id = parse_log_id(&log_id);
    let timestamp = DateTime::from_timestamp_micros(ts as i64)
        .expect("Should convert DateTime correctly")
        .naive_utc();
    pg_client.execute("INSERT INTO lnv1_incoming_payment_started (log_id, ts, federation_id, federation_name, contract_id, contract_amount, invoice_amount, operation_id, payment_hash) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        &[&log_id, &timestamp, &federation_id.to_string(), &federation_name, &contract_id, &contract_amount, &invoice_amount, &operation_id, &payment_hash]).await?;
    Ok(())
}

async fn insert_outgoing_payment_failed(
    pg_client: &Client,
    log_id: &EventLogId,
    ts: u64,
    federation_id: FederationId,
    federation_name: String,
    contract_id: String,
    contract_amount: i64,
    gateway_key: String,
    payment_hash: String,
    timelock: i64,
    user_key: String,
    error_reason: Option<String>,
) -> anyhow::Result<()> {
    let log_id = parse_log_id(&log_id);
    let timestamp = DateTime::from_timestamp_micros(ts as i64)
        .expect("Should convert DateTime correctly")
        .naive_utc();
    pg_client.execute("INSERT INTO lnv1_outgoing_payment_failed (log_id, ts, federation_id, federation_name, contract_id, contract_amount, gateway_key, payment_hash, timelock, user_key, error_reason) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)", 
    &[&log_id, &timestamp, &federation_id.to_string(), &federation_name, &contract_id, &contract_amount, &gateway_key, &payment_hash, &timelock, &user_key, &error_reason]).await?;
    Ok(())
}

async fn insert_outgoing_payment_succeeded(
    pg_client: &Client,
    log_id: &EventLogId,
    ts: u64,
    federation_id: FederationId,
    federation_name: String,
    contract_id: String,
    contract_amount: i64,
    gateway_key: String,
    payment_hash: String,
    timelock: i64,
    user_key: String,
    preimage: String,
) -> anyhow::Result<()> {
    let log_id = parse_log_id(&log_id);
    let timestamp = DateTime::from_timestamp_micros(ts as i64)
        .expect("Should convert DateTime correctly")
        .naive_utc();
    pg_client.execute("INSERT INTO lnv1_outgoing_payment_succeeded (log_id, ts, federation_id, federation_name, contract_id, contract_amount, gateway_key, payment_hash, timelock, user_key, preimage) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)", 
    &[&log_id, &timestamp, &federation_id.to_string(), &federation_name, &contract_id, &contract_amount, &gateway_key, &payment_hash, &timelock, &user_key, &preimage]).await?;
    Ok(())
}

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

async fn connect_pg(db_host: String, db_user: String, db_password: String, db_name: String) -> anyhow::Result<Client> {
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
