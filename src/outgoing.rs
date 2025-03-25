use chrono::DateTime;
use fedimint_core::{anyhow, config::FederationId};
use fedimint_eventlog::EventLogId;
use serde::{de, Deserialize};
use serde_json::Value;
use tokio_postgres::Client;

use crate::parse_log_id;


#[derive(Debug, Clone)]
pub(crate) struct Lnv1OutgoingPaymentStarted {
    contract_id: String,
    amount: i64,
    operation_id: String,
}

impl<'de> Deserialize<'de> for Lnv1OutgoingPaymentStarted {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        let value = Value::deserialize(deserializer)?;
        
        let contract_id = value["contract_id"]
            .as_str()
            .ok_or_else(|| de::Error::missing_field("contract_id"))?
            .to_string();
        let operation_id = value["operation_id"]
            .as_str()
            .ok_or_else(|| de::Error::missing_field("operation_id"))?
            .to_string();
        let amount = value["invoice_amount"]
            .as_u64()
            .ok_or_else(|| de::Error::missing_field("invoice_amount"))?
            as i64;
        
        Ok(Lnv1OutgoingPaymentStarted {
            contract_id,
            amount,
            operation_id,
        })
    }
}

impl Lnv1OutgoingPaymentStarted {
    pub async fn insert(&self, pg_client: &Client, log_id: &EventLogId, timestamp: u64, federation_id: &FederationId, federation_name: String) -> anyhow::Result<()> {
        let log_id = parse_log_id(&log_id);
        let timestamp = DateTime::from_timestamp_micros(timestamp as i64)
            .expect("Should convert DateTime correctly")
            .naive_utc();
        pg_client.execute("INSERT INTO lnv1_outgoing_payment_started (log_id, ts, federation_id, federation_name, contract_id, invoice_amount, operation_id) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        &[&log_id, &timestamp, &federation_id.to_string(), &federation_name, &self.contract_id, &(self.amount as i64), &self.operation_id]).await?;
        Ok(())
    }
}