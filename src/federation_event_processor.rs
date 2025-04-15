use std::fmt;

use fedimint_core::{anyhow, config::FederationId};
use fedimint_eventlog::{EventKind, EventLogId};
use fedimint_gateway_client::GatewayRpcClient;
use fedimint_gateway_common::{FederationInfo, PaymentLogPayload};
use serde_json::Value;
use tokio_postgres::Client;
use tracing::warn;

use crate::{
    CompleteLightningPaymentSucceeded, DbConnection, LNv1IncomingPaymentFailed,
    LNv1IncomingPaymentStarted, LNv1IncomingPaymentSucceeded, LNv1OutgoingPaymentFailed,
    LNv1OutgoingPaymentStarted, LNv1OutgoingPaymentSucceeded, TelegramClient, parse_log_id,
};

pub(crate) struct FederationEventProcessor {
    federation_id: FederationId,
    federation_name: String,
    max_log_id: i64,
    pg_client: Client,
    gw_client: GatewayRpcClient,
    telegram_client: TelegramClient,
    outgoing_payment_started_count: u64,
    outgoing_payment_succeeded_count: u64,
    outgoing_payment_failed_count: u64,
    incoming_payment_started_count: u64,
    incoming_payment_succeeded_count: u64,
    incoming_payment_failed_count: u64,
    complete_lightning_payment_succeeded_count: u64,
}

impl fmt::Display for FederationEventProcessor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Federation: {}\n\
            Outgoing Payments - Succeeded: {}, Failed: {}\n\
            Incoming Payments - Succeeded: {}, Failed: {}\n\n",
            self.federation_name,
            self.outgoing_payment_succeeded_count,
            self.outgoing_payment_failed_count,
            self.incoming_payment_succeeded_count,
            self.incoming_payment_failed_count,
        )
    }
}

impl FederationEventProcessor {
    pub async fn new(
        fed_info: FederationInfo,
        db_conn: DbConnection,
        gw_client: GatewayRpcClient,
        telegram_client: TelegramClient,
    ) -> anyhow::Result<FederationEventProcessor> {
        let pg_client = db_conn.connect().await?;
        let max_log_id = Self::get_max_log_id(&pg_client, fed_info.federation_id).await?;
        Ok(Self {
            federation_id: fed_info.federation_id,
            federation_name: fed_info
                .federation_name
                .expect("No federation name provided"),
            max_log_id,
            pg_client,
            gw_client,
            telegram_client,
            outgoing_payment_started_count: 0,
            outgoing_payment_succeeded_count: 0,
            outgoing_payment_failed_count: 0,
            incoming_payment_started_count: 0,
            incoming_payment_succeeded_count: 0,
            incoming_payment_failed_count: 0,
            complete_lightning_payment_succeeded_count: 0,
        })
    }

    async fn get_max_log_id(
        pg_client: &Client,
        federation_id: FederationId,
    ) -> anyhow::Result<i64> {
        let query = "
            SELECT MAX(log_id)
            FROM (
                SELECT log_id FROM lnv1_outgoing_payment_started WHERE federation_id = $1
                UNION ALL
                SELECT log_id FROM lnv1_outgoing_payment_succeeded WHERE federation_id = $1
                UNION ALL
                SELECT log_id FROM lnv1_outgoing_payment_failed WHERE federation_id = $1
                UNION ALL
                SELECT log_id FROM lnv1_incoming_payment_started WHERE federation_id = $1
                UNION ALL
                SELECT log_id FROM lnv1_incoming_payment_succeeded WHERE federation_id = $1
                UNION ALL
                SELECT log_id FROM lnv1_incoming_payment_failed WHERE federation_id = $1
                UNION ALL
                SELECT log_id FROM lnv1_complete_lightning_payment_succeeded WHERE federation_id = $1
            ) AS combined_log_ids
        ";

        let rows = pg_client
            .query(query, &[&federation_id.to_string()])
            .await?;
        if let Some(row) = rows.get(0) {
            let max_log_id: Option<i64> = row.get(0);
            if let Some(max_log_id) = max_log_id {
                return Ok(max_log_id);
            }
        }

        Ok(0)
    }

    pub async fn process_events(&mut self) -> anyhow::Result<()> {
        let payment_log = self
            .gw_client
            .payment_log(PaymentLogPayload {
                end_position: None,
                pagination_size: usize::MAX,
                federation_id: self.federation_id,
                event_kinds: vec![],
            })
            .await?;

        for entry in payment_log.0 {
            if parse_log_id(&entry.event_id) <= self.max_log_id {
                break;
            }

            match entry.module {
                Some((module, _)) if module.as_str() == "ln" => {
                    self.handle_lnv1(
                        entry.event_id,
                        entry.event_kind,
                        entry.timestamp,
                        entry.value,
                    )
                    .await?;
                }
                Some((module, _)) => {
                    warn!(module = %module, "Unsupported module");
                    self.telegram_client
                        .send_telegram_message(format!("Found unsupported module: {module}"))
                        .await;
                }
                None => {
                    warn!("No module provided");
                    self.telegram_client
                        .send_telegram_message("Found event without a module".to_string())
                        .await;
                }
            }
        }

        Ok(())
    }

    async fn handle_lnv1(
        &mut self,
        log_id: EventLogId,
        kind: EventKind,
        timestamp: u64,
        value: Value,
    ) -> anyhow::Result<()> {
        let kind = Self::parse_event_kind(format!("{kind:?}"));
        match kind.as_str() {
            "outgoing-payment-started" => {
                let outgoing_payment_started_event: LNv1OutgoingPaymentStarted =
                    serde_json::from_value(value).expect("Could not parse event");
                outgoing_payment_started_event
                    .insert(
                        &self.pg_client,
                        &log_id,
                        timestamp,
                        &self.federation_id,
                        self.federation_name.clone(),
                    )
                    .await?;
                self.outgoing_payment_started_count += 1;
            }
            "outgoing-payment-succeeded" => {
                let outgoing_payment_succeeded_event: LNv1OutgoingPaymentSucceeded =
                    serde_json::from_value(value).expect("Could not parse event");
                outgoing_payment_succeeded_event
                    .insert(
                        &self.pg_client,
                        &log_id,
                        timestamp,
                        &self.federation_id,
                        self.federation_name.clone(),
                    )
                    .await?;
                self.outgoing_payment_succeeded_count += 1;
            }
            "outgoing-payment-failed" => {
                let outgoing_payment_failed_event: LNv1OutgoingPaymentFailed =
                    serde_json::from_value(value).expect("Could not parse event");
                outgoing_payment_failed_event
                    .insert(
                        &self.pg_client,
                        &log_id,
                        timestamp,
                        &self.federation_id,
                        self.federation_name.clone(),
                    )
                    .await?;
                self.outgoing_payment_failed_count += 1;
            }
            "incoming-payment-started" => {
                let incoming_payment_started_event: LNv1IncomingPaymentStarted =
                    serde_json::from_value(value).expect("Could not parse event");
                incoming_payment_started_event
                    .insert(
                        &self.pg_client,
                        &log_id,
                        timestamp,
                        &self.federation_id,
                        self.federation_name.clone(),
                    )
                    .await?;
                self.incoming_payment_started_count += 1;
            }
            "incoming-payment-succeeded" => {
                let incoming_payment_succeeded_event: LNv1IncomingPaymentSucceeded =
                    serde_json::from_value(value).expect("Could not parse event");
                incoming_payment_succeeded_event
                    .insert(
                        &self.pg_client,
                        &log_id,
                        timestamp,
                        &self.federation_id,
                        self.federation_name.clone(),
                    )
                    .await?;
                self.incoming_payment_succeeded_count += 1;
            }
            "incoming-payment-failed" => {
                let incoming_payment_failed_event: LNv1IncomingPaymentFailed =
                    serde_json::from_value(value).expect("Could not parse event");
                incoming_payment_failed_event
                    .insert(
                        &self.pg_client,
                        &log_id,
                        timestamp,
                        &self.federation_id,
                        self.federation_name.clone(),
                    )
                    .await?;
                self.incoming_payment_failed_count += 1;
            }
            "complete-lightning-payment-succeeded" => {
                let complete_lightning_payment_succeeded_event: CompleteLightningPaymentSucceeded =
                    serde_json::from_value(value).expect("Could not parse event");
                complete_lightning_payment_succeeded_event
                    .insert(
                        &self.pg_client,
                        &log_id,
                        timestamp,
                        &self.federation_id,
                        self.federation_name.clone(),
                    )
                    .await?;
                self.complete_lightning_payment_succeeded_count += 1;
            }
            event => {
                warn!(?event, "Unrecognized event");
            }
        }

        Ok(())
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
}
