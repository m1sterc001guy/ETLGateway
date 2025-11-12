use std::time::{Duration, UNIX_EPOCH};

use clap::Parser;
use federation_event_processor::FederationEventProcessor;
use fedimint_core::{anyhow, time::now, util::SafeUrl};
use fedimint_eventlog::EventLogId;
use fedimint_gateway_client::GatewayRpcClient;
use fedimint_gateway_common::PaymentSummaryPayload;
use fedimint_logging::TracingSetup;
use incoming::{
    LNv1CompleteLightningPaymentSucceeded, LNv1IncomingPaymentFailed, LNv1IncomingPaymentStarted,
    LNv1IncomingPaymentSucceeded,
};
use outgoing::{
    LNv1OutgoingPaymentFailed, LNv1OutgoingPaymentStarted, LNv1OutgoingPaymentSucceeded,
};
use serde_json::json;
use tokio_postgres::{Client, NoTls};
use tracing::{error, info};

mod federation_event_processor;
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

    #[arg(long = "gateway-epoch", env = "GW_EPOCH")]
    gateway_epoch: i32,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    TracingSetup::default().init()?;
    let opts = GatewayETLOpts::parse();
    let conn = DbConnection::from_opts(&opts);

    let telegram_client = TelegramClient::from_opts(&opts);
    let client = GatewayRpcClient::new(opts.gateway_addr.clone(), Some(opts.password.clone()));
    let info = client.get_info().await?;
    let mut message = String::new();
    let now = now();
    let now_millis = now
        .duration_since(UNIX_EPOCH)
        .expect("Before unix epoch")
        .as_millis()
        .try_into()?;
    let one_day_ago = now
        .checked_sub(Duration::from_secs(60 * 60 * 24))
        .expect("Before unix epoch");
    let one_day_ago_millis = one_day_ago
        .duration_since(UNIX_EPOCH)
        .expect("Before unix epoch")
        .as_millis()
        .try_into()?;
    let summary = client
        .payment_summary(PaymentSummaryPayload {
            start_millis: one_day_ago_millis,
            end_millis: now_millis,
        })
        .await?;

    message += "===========24 HOUR SUMMARY===========\n";
    message += format!(
        "Outgoing Average Latency: {}ms\n",
        summary
            .outgoing
            .average_latency
            .unwrap_or_default()
            .as_millis()
    )
    .as_str();
    message += format!(
        "Outgoing Median Latency: {}ms\n",
        summary
            .outgoing
            .median_latency
            .unwrap_or_default()
            .as_millis()
    )
    .as_str();
    message += format!("Outgoing Fees: {}\n", summary.outgoing.total_fees).as_str();
    message += format!(
        "Incoming Average Latency: {}ms\n",
        summary
            .incoming
            .average_latency
            .unwrap_or_default()
            .as_millis()
    )
    .as_str();
    message += format!(
        "Incoming Median Latency: {}ms\n",
        summary
            .incoming
            .median_latency
            .unwrap_or_default()
            .as_millis()
    )
    .as_str();
    message += format!("Incoming Fees: {}\n\n", summary.incoming.total_fees).as_str();

    for fed_info in info.federations {
        let client = GatewayRpcClient::new(opts.gateway_addr.clone(), Some(opts.password.clone()));
        let mut processor = FederationEventProcessor::new(
            fed_info,
            conn.clone(),
            client,
            telegram_client.clone(),
            opts.gateway_epoch,
        )
        .await?;
        processor.process_events().await?;

        message += format!("{processor}").as_str();
    }

    info!(message);
    //telegram_client.send_telegram_message(message).await;
    Ok(())
}

#[derive(Debug, Clone)]
struct TelegramClient {
    bot_token: String,
    chat_id: String,
    client: reqwest::Client,
}

impl TelegramClient {
    fn from_opts(opts: &GatewayETLOpts) -> TelegramClient {
        TelegramClient {
            bot_token: opts.bot_token.clone(),
            chat_id: opts.chat_id.clone(),
            client: reqwest::Client::new(),
        }
    }

    async fn send_telegram_message(&self, message: String) {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);

        let res = self
            .client
            .post(&url)
            .json(&json!({
                "chat_id": self.chat_id,
                "text": message,
            }))
            .send()
            .await;

        match res {
            Ok(response) => {
                info!(
                    "Successfully sent Telegram message! Response: {:?}",
                    response
                );
            }
            Err(err) => {
                error!("Error sending message: {}", err);
            }
        }
    }
}

#[derive(Debug, Clone)]
struct DbConnection {
    db_host: String,
    db_user: String,
    db_password: String,
    db_name: String,
}

impl DbConnection {
    fn from_opts(opts: &GatewayETLOpts) -> DbConnection {
        DbConnection {
            db_host: opts.db_host.clone(),
            db_user: opts.db_user.clone(),
            db_password: opts.db_password.clone(),
            db_name: opts.db_name.clone(),
        }
    }

    async fn connect(&self) -> anyhow::Result<Client> {
        let (pg_client, pg_connection) = tokio_postgres::connect(
            format!(
                "host={} user={} password={} dbname={}",
                self.db_host, self.db_user, self.db_password, self.db_name
            )
            .as_str(),
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
