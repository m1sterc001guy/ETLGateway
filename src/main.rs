use clap::Parser;
use federation_event_processor::FederationEventProcessor;
use fedimint_core::{anyhow, util::SafeUrl};
use fedimint_eventlog::EventLogId;
use fedimint_logging::TracingSetup;
use incoming::{
    CompleteLightningPaymentSucceeded, LNv1IncomingPaymentFailed, LNv1IncomingPaymentStarted,
    LNv1IncomingPaymentSucceeded,
};
use ln_gateway::rpc::rpc_client::GatewayRpcClient;
use outgoing::{
    LNv1OutgoingPaymentFailed, LNv1OutgoingPaymentStarted, LNv1OutgoingPaymentSucceeded,
};
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    TracingSetup::default().init()?;
    let opts = GatewayETLOpts::parse();
    let conn = DbConnection::from_opts(&opts);

    let client = GatewayRpcClient::new(opts.gateway_addr.clone(), Some(opts.password.clone()));
    let info = client.get_info().await?;
    for fed_info in info.federations {
        let client = GatewayRpcClient::new(opts.gateway_addr.clone(), Some(opts.password.clone()));
        let mut processor = FederationEventProcessor::new(fed_info, conn.clone(), client).await?;
        processor.process_events().await?;

        info!("{processor}");
    }

    Ok(())
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
        info!("Connecting to database...");
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
