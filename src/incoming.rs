use chrono::DateTime;
use fedimint_core::{anyhow, config::FederationId};
use fedimint_eventlog::EventLogId;
use serde::Deserialize;
use serde_json::Value;
use tokio_postgres::Client;

use crate::{outgoing::LNv2PaymentImage, parse_log_id};

#[derive(Debug, Clone)]
pub(crate) struct LNv2IncomingPaymentStarted {
    incoming_contract_commitment: LNv2IncomingContractCommitment,
    invoice_amount: i64,
    operation_start: i64,
}

impl<'de> Deserialize<'de> for LNv2IncomingPaymentStarted {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let incoming_contract_commitment: LNv2IncomingContractCommitment =
            serde_json::from_value(value["incoming_contract_commitment"].clone())
                .expect("Could not parse LNv2PaymentImage");
        let invoice_amount = value["invoice_amount"]
            .as_i64()
            .expect("amount should be present");
        let operation_start = value["operation_start"]
            .as_i64()
            .expect("amount should be present");

        Ok(Self {
            incoming_contract_commitment,
            invoice_amount,
            operation_start,
        })
    }
}

impl LNv2IncomingPaymentStarted {
    pub async fn insert(
        &self,
        pg_client: &Client,
        log_id: &EventLogId,
        timestamp: u64,
        federation_id: &FederationId,
        federation_name: String,
        gateway_epoch: i32,
    ) -> anyhow::Result<()> {
        let log_id = parse_log_id(&log_id);
        let timestamp = DateTime::from_timestamp_micros(timestamp as i64)
            .expect("Should convert DateTime correctly")
            .naive_utc();
        let operation_start = DateTime::from_timestamp_micros(self.operation_start as i64)
            .expect("Should convert DateTime correctly")
            .naive_utc();
        pg_client.execute("INSERT INTO lnv2_incoming_payment_started (log_id, ts, federation_id, federation_name, gateway_epoch, amount, claim_pk, ephemeral_pk, expiration, payment_image, refund_pk, invoice_amount, operation_start) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
        &[&log_id, &timestamp, &federation_id.to_string(), &federation_name, &gateway_epoch, &self.incoming_contract_commitment.amount, &self.incoming_contract_commitment.claim_pk, &self.incoming_contract_commitment.ephemeral_pk, &self.incoming_contract_commitment.expiration, &self.incoming_contract_commitment.payment_image.hash, &self.incoming_contract_commitment.refund_pk, &self.invoice_amount, &operation_start]).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LNv2IncomingContractCommitment {
    amount: i64,
    claim_pk: String,
    ephemeral_pk: String,
    expiration: i64,
    payment_image: LNv2PaymentImage,
    refund_pk: String,
}

impl<'de> Deserialize<'de> for LNv2IncomingContractCommitment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let amount = value["amount"].as_i64().expect("amount should be present");
        let claim_pk = value["claim_pk"]
            .as_str()
            .expect("Should be present")
            .to_string();
        let ephemeral_pk = value["ephemeral_pk"]
            .as_str()
            .expect("Should be present")
            .to_string();
        let expiration = value["expiration"]
            .as_i64()
            .expect("amount should be present");
        let payment_image: LNv2PaymentImage =
            serde_json::from_value(value["payment_image"].clone())
                .expect("Could not parse LNv2PaymentImage");
        let refund_pk = value["refund_pk"]
            .as_str()
            .expect("Should be present")
            .to_string();

        Ok(Self {
            amount,
            claim_pk,
            ephemeral_pk,
            expiration,
            payment_image,
            refund_pk,
        })
    }
}

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
        gateway_epoch: i32,
    ) -> anyhow::Result<()> {
        let log_id = parse_log_id(&log_id);
        let timestamp = DateTime::from_timestamp_micros(timestamp as i64)
            .expect("Should convert DateTime correctly")
            .naive_utc();
        pg_client.execute("INSERT INTO lnv1_incoming_payment_started (log_id, ts, federation_id, federation_name, contract_id, contract_amount, invoice_amount, operation_id, payment_hash, gateway_epoch) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        &[&log_id, &timestamp, &federation_id.to_string(), &federation_name, &self.contract_id, &self.contract_amount, &self.invoice_amount, &self.operation_id, &self.payment_hash, &gateway_epoch]).await?;
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
        gateway_epoch: i32,
    ) -> anyhow::Result<()> {
        let log_id = parse_log_id(&log_id);
        let timestamp = DateTime::from_timestamp_micros(timestamp as i64)
            .expect("Should convert DateTime correctly")
            .naive_utc();
        pg_client.execute("INSERT INTO lnv1_incoming_payment_succeeded (log_id, ts, federation_id, federation_name, payment_hash, preimage, gateway_epoch) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    &[&log_id, &timestamp, &federation_id.to_string(), &federation_name, &self.payment_hash, &self.preimage, &gateway_epoch]).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LNv2IncomingPaymentSucceeded {
    payment_image: LNv2PaymentImage,
}

impl<'de> Deserialize<'de> for LNv2IncomingPaymentSucceeded {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let payment_image: LNv2PaymentImage =
            serde_json::from_value(value["payment_image"].clone())
                .expect("Could not parse payment_image");
        Ok(Self { payment_image })
    }
}

impl LNv2IncomingPaymentSucceeded {
    pub async fn insert(
        &self,
        pg_client: &Client,
        log_id: &EventLogId,
        timestamp: u64,
        federation_id: &FederationId,
        federation_name: String,
        gateway_epoch: i32,
    ) -> anyhow::Result<()> {
        let log_id = parse_log_id(&log_id);
        let timestamp = DateTime::from_timestamp_micros(timestamp as i64)
            .expect("Should convert DateTime correctly")
            .naive_utc();
        pg_client.execute("INSERT INTO lnv2_incoming_payment_succeeded (log_id, ts, federation_id, federation_name, gateway_epoch, payment_image) VALUES ($1, $2, $3, $4, $5, $6)",
    &[&log_id, &timestamp, &federation_id.to_string(), &federation_name, &gateway_epoch, &self.payment_image.hash]).await?;
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
        gateway_epoch: i32,
    ) -> anyhow::Result<()> {
        let log_id = parse_log_id(&log_id);
        let timestamp = DateTime::from_timestamp_micros(timestamp as i64)
            .expect("Should convert DateTime correctly")
            .naive_utc();
        pg_client.execute("INSERT INTO lnv1_incoming_payment_failed (log_id, ts, federation_id, federation_name, payment_hash, error_reason, gateway_epoch) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    &[&log_id, &timestamp, &federation_id.to_string(), &federation_name, &self.payment_hash, &self.error, &gateway_epoch]).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LNv2IncomingPaymentFailed {
    payment_image: LNv2PaymentImage,
    error: String,
}

impl<'de> Deserialize<'de> for LNv2IncomingPaymentFailed {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let payment_image: LNv2PaymentImage =
            serde_json::from_value(value["payment_image"].clone())
                .expect("Could not parse payment_image");
        let error = value["error"]
            .as_str()
            .expect("Should be present")
            .to_string();

        Ok(Self {
            payment_image,
            error,
        })
    }
}

impl LNv2IncomingPaymentFailed {
    pub async fn insert(
        &self,
        pg_client: &Client,
        log_id: &EventLogId,
        timestamp: u64,
        federation_id: &FederationId,
        federation_name: String,
        gateway_epoch: i32,
    ) -> anyhow::Result<()> {
        let log_id = parse_log_id(&log_id);
        let timestamp = DateTime::from_timestamp_micros(timestamp as i64)
            .expect("Should convert DateTime correctly")
            .naive_utc();
        pg_client.execute("INSERT INTO lnv2_incoming_payment_failed (log_id, ts, federation_id, federation_name, gateway_epoch, payment_image, error) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    &[&log_id, &timestamp, &federation_id.to_string(), &federation_name, &gateway_epoch, &self.payment_image.hash, &self.error]).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LNv1CompleteLightningPaymentSucceeded {
    payment_hash: String,
}

impl<'de> Deserialize<'de> for LNv1CompleteLightningPaymentSucceeded {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        let payment_hash = value["payment_hash"]
            .as_str()
            .expect("Should be present")
            .to_string();

        Ok(LNv1CompleteLightningPaymentSucceeded { payment_hash })
    }
}

impl LNv1CompleteLightningPaymentSucceeded {
    pub async fn insert(
        &self,
        pg_client: &Client,
        log_id: &EventLogId,
        timestamp: u64,
        federation_id: &FederationId,
        federation_name: String,
        gateway_epoch: i32,
    ) -> anyhow::Result<()> {
        let log_id = parse_log_id(&log_id);
        let timestamp = DateTime::from_timestamp_micros(timestamp as i64)
            .expect("Should convert DateTime correctly")
            .naive_utc();
        pg_client.execute("INSERT INTO lnv1_complete_lightning_payment_succeeded (log_id, ts, federation_id, federation_name, payment_hash, gateway_epoch) VALUES ($1, $2, $3, $4, $5, $6)",
    &[&log_id, &timestamp, &federation_id.to_string(), &federation_name, &self.payment_hash, &gateway_epoch]).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LNv2CompleteLightningPaymentSucceeded {
    payment_image: LNv2PaymentImage,
}

impl<'de> Deserialize<'de> for LNv2CompleteLightningPaymentSucceeded {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let payment_image: LNv2PaymentImage =
            serde_json::from_value(value["payment_image"].clone())
                .expect("Could not parse payment_image");
        Ok(Self { payment_image })
    }
}

impl LNv2CompleteLightningPaymentSucceeded {
    pub async fn insert(
        &self,
        pg_client: &Client,
        log_id: &EventLogId,
        timestamp: u64,
        federation_id: &FederationId,
        federation_name: String,
        gateway_epoch: i32,
    ) -> anyhow::Result<()> {
        let log_id = parse_log_id(&log_id);
        let timestamp = DateTime::from_timestamp_micros(timestamp as i64)
            .expect("Should convert DateTime correctly")
            .naive_utc();
        pg_client.execute("INSERT INTO lnv2_complete_lightning_payment_succeeded (log_id, ts, federation_id, federation_name, gateway_epoch, payment_image) VALUES ($1, $2, $3, $4, $5, $6)",
    &[&log_id, &timestamp, &federation_id.to_string(), &federation_name, &gateway_epoch, &self.payment_image.hash]).await?;
        Ok(())
    }
}
