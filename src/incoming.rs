use chrono::DateTime;
use fedimint_core::{anyhow, config::FederationId};
use fedimint_eventlog::EventLogId;
use serde::Deserialize;
use serde_json::Value;
use tokio_postgres::Client;

use crate::parse_log_id;

#[derive(Debug, Clone)]
pub(crate) struct LNv1IncomingPaymentStarted {
    contract_id: String,
    contract_amount: i64,
    invoice_amount: i64,
    operation_id: String,
    payment_hash: String,
}

impl<'de> Deserialize<'de> for LNv1IncomingPaymentStarted {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        let contract_id = value["contract_id"]
            .as_str()
            .expect("Should be present")
            .to_string();
        let contract_amount = value["contract_amount"]
            .as_i64()
            .expect("contract amount should be present");
        let invoice_amount = value["invoice_amount"]
            .as_i64()
            .expect("invoice amount should be present");
        let operation_id = value["operation_id"]
            .as_str()
            .expect("Should be present")
            .to_string();
        let payment_hash = value["payment_hash"]
            .as_str()
            .expect("Should be present")
            .to_string();

        Ok(LNv1IncomingPaymentStarted {
            contract_id,
            contract_amount,
            invoice_amount,
            operation_id,
            payment_hash,
        })
    }
}

impl LNv1IncomingPaymentStarted {
    pub async fn insert(
        &self,
        pg_client: &Client,
        log_id: &EventLogId,
        timestamp: u64,
        federation_id: &FederationId,
        federation_name: String,
    ) -> anyhow::Result<()> {
        let log_id = parse_log_id(&log_id);
        let timestamp = DateTime::from_timestamp_micros(timestamp as i64)
            .expect("Should convert DateTime correctly")
            .naive_utc();
        pg_client.execute("INSERT INTO lnv1_incoming_payment_started (log_id, ts, federation_id, federation_name, contract_id, contract_amount, invoice_amount, operation_id, payment_hash) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        &[&log_id, &timestamp, &federation_id.to_string(), &federation_name, &self.contract_id, &self.contract_amount, &self.invoice_amount, &self.operation_id, &self.payment_hash]).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LNv1IncomingPaymentSucceeded {
    payment_hash: String,
    preimage: String,
}

impl<'de> Deserialize<'de> for LNv1IncomingPaymentSucceeded {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        let payment_hash = value["payment_hash"]
            .as_str()
            .expect("Should be present")
            .to_string();
        let preimage = value["preimage"]
            .as_str()
            .expect("Should be present")
            .to_string();

        Ok(LNv1IncomingPaymentSucceeded {
            payment_hash,
            preimage,
        })
    }
}

impl LNv1IncomingPaymentSucceeded {
    pub async fn insert(
        &self,
        pg_client: &Client,
        log_id: &EventLogId,
        timestamp: u64,
        federation_id: &FederationId,
        federation_name: String,
    ) -> anyhow::Result<()> {
        let log_id = parse_log_id(&log_id);
        let timestamp = DateTime::from_timestamp_micros(timestamp as i64)
            .expect("Should convert DateTime correctly")
            .naive_utc();
        pg_client.execute("INSERT INTO lnv1_incoming_payment_succeeded (log_id, ts, federation_id, federation_name, payment_hash, preimage) VALUES ($1, $2, $3, $4, $5, $6)",
    &[&log_id, &timestamp, &federation_id.to_string(), &federation_name, &self.payment_hash, &self.preimage]).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LNv1IncomingPaymentFailed {
    payment_hash: String,
    error: String,
}

impl<'de> Deserialize<'de> for LNv1IncomingPaymentFailed {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        let payment_hash = value["payment_hash"]
            .as_str()
            .expect("Should be present")
            .to_string();
        let error = value["error"]
            .as_str()
            .expect("Should be present")
            .to_string();

        Ok(LNv1IncomingPaymentFailed {
            payment_hash,
            error,
        })
    }
}

impl LNv1IncomingPaymentFailed {
    pub async fn insert(
        &self,
        pg_client: &Client,
        log_id: &EventLogId,
        timestamp: u64,
        federation_id: &FederationId,
        federation_name: String,
    ) -> anyhow::Result<()> {
        let log_id = parse_log_id(&log_id);
        let timestamp = DateTime::from_timestamp_micros(timestamp as i64)
            .expect("Should convert DateTime correctly")
            .naive_utc();
        pg_client.execute("INSERT INTO lnv1_incoming_payment_failed (log_id, ts, federation_id, federation_name, payment_hash, error_reason) VALUES ($1, $2, $3, $4, $5, $6)",
    &[&log_id, &timestamp, &federation_id.to_string(), &federation_name, &self.payment_hash, &self.error]).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CompleteLightningPaymentSucceeded {
    payment_hash: String,
}

impl<'de> Deserialize<'de> for CompleteLightningPaymentSucceeded {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        let payment_hash = value["payment_hash"]
            .as_str()
            .expect("Should be present")
            .to_string();

        Ok(CompleteLightningPaymentSucceeded { payment_hash })
    }
}

impl CompleteLightningPaymentSucceeded {
    pub async fn insert(
        &self,
        pg_client: &Client,
        log_id: &EventLogId,
        timestamp: u64,
        federation_id: &FederationId,
        federation_name: String,
    ) -> anyhow::Result<()> {
        let log_id = parse_log_id(&log_id);
        let timestamp = DateTime::from_timestamp_micros(timestamp as i64)
            .expect("Should convert DateTime correctly")
            .naive_utc();
        pg_client.execute("INSERT INTO lnv1_complete_lightning_payment_succeeded (log_id, ts, federation_id, federation_name, payment_hash) VALUES ($1, $2, $3, $4, $5)",
    &[&log_id, &timestamp, &federation_id.to_string(), &federation_name, &self.payment_hash]).await?;
        Ok(())
    }
}
