use chrono::DateTime;
use fedimint_core::{anyhow, config::FederationId};
use fedimint_eventlog::EventLogId;
use serde::{Deserialize, de};
use serde_json::Value;
use tokio_postgres::Client;

use crate::parse_log_id;

#[derive(Debug, Clone)]
pub(crate) struct LNv2OutgoingPaymentStarted {
    invoice_amount: i64,
    max_delay: i64,
    min_contract_amount: i64,
    operation_start: i64,
    outgoing_contract: LNv2OutgoingContract,
}

impl<'de> Deserialize<'de> for LNv2OutgoingPaymentStarted {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        let invoice_amount = value["invoice_amount"]
            .as_u64()
            .ok_or_else(|| de::Error::missing_field("invoice_amount"))?
            as i64;
        let max_delay = value["max_delay"]
            .as_u64()
            .ok_or_else(|| de::Error::missing_field("max_delay"))? as i64;
        let min_contract_amount = value["min_contract_amount"]
            .as_u64()
            .ok_or_else(|| de::Error::missing_field("min_contract_amount"))?
            as i64;
        let operation_start = value["operation_start"]
            .as_u64()
            .ok_or_else(|| de::Error::missing_field("operation_start"))?
            as i64;
        let outgoing_contract: LNv2OutgoingContract =
            serde_json::from_value(value["outgoing_contract"].clone())
                .map_err(|e| de::Error::custom(e.to_string()))?;

        Ok(Self {
            invoice_amount,
            max_delay,
            min_contract_amount,
            operation_start,
            outgoing_contract,
        })
    }
}

impl LNv2OutgoingPaymentStarted {
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
        let operation_start = DateTime::from_timestamp_micros(self.operation_start)
            .expect("Should convert DateTime correctly")
            .naive_utc();
        pg_client.execute("INSERT INTO lnv2_outgoing_payment_started (log_id, ts, federation_id, federation_name, gateway_epoch, invoice_amount, max_delay, min_contract_amount, operation_start, amount, claim_pk, ephemeral_pk, expiration, payment_image, refund_pk) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)",
        &[&log_id, &timestamp, &federation_id.to_string(), &federation_name, &gateway_epoch, &self.invoice_amount, &self.max_delay, &self.min_contract_amount, &operation_start, &self.outgoing_contract.amount, &self.outgoing_contract.claim_pk, &self.outgoing_contract.ephemeral_pk, &self.outgoing_contract.expiration, &self.outgoing_contract.payment_image.hash, &self.outgoing_contract.refund_pk]).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LNv2OutgoingContract {
    amount: i64,
    claim_pk: String,
    ephemeral_pk: String,
    expiration: i64,
    payment_image: LNv2PaymentImage,
    refund_pk: String,
}

impl<'de> Deserialize<'de> for LNv2OutgoingContract {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        let amount = value["amount"]
            .as_u64()
            .ok_or_else(|| de::Error::missing_field("amount"))? as i64;
        let claim_pk = value["claim_pk"]
            .as_str()
            .ok_or_else(|| de::Error::missing_field("claim_pk"))?
            .to_string();
        let ephemeral_pk = value["ephemeral_pk"]
            .as_str()
            .ok_or_else(|| de::Error::missing_field("ephemeral_pk"))?
            .to_string();
        let expiration = value["expiration"]
            .as_u64()
            .ok_or_else(|| de::Error::missing_field("expiration"))? as i64;
        let payment_image: LNv2PaymentImage =
            serde_json::from_value(value["payment_image"].clone())
                .map_err(|e| de::Error::custom(e.to_string()))?;
        let refund_pk = value["refund_pk"]
            .as_str()
            .ok_or_else(|| de::Error::missing_field("refund_pk"))?
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
pub(crate) struct LNv2PaymentImage {
    pub(crate) hash: String,
}

impl<'de> Deserialize<'de> for LNv2PaymentImage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        let hash = value["Hash"]
            .as_str()
            .ok_or_else(|| de::Error::missing_field("Hash"))?
            .to_string();
        Ok(Self { hash })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LNv1OutgoingPaymentStarted {
    contract_id: String,
    amount: i64,
    operation_id: String,
}

impl<'de> Deserialize<'de> for LNv1OutgoingPaymentStarted {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
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
            .ok_or_else(|| de::Error::missing_field("invoice_amount"))? as i64;

        Ok(LNv1OutgoingPaymentStarted {
            contract_id,
            amount,
            operation_id,
        })
    }
}

impl LNv1OutgoingPaymentStarted {
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
        pg_client.execute("INSERT INTO lnv1_outgoing_payment_started (log_id, ts, federation_id, federation_name, contract_id, invoice_amount, operation_id, gateway_epoch) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        &[&log_id, &timestamp, &federation_id.to_string(), &federation_name, &self.contract_id, &(self.amount as i64), &self.operation_id, &gateway_epoch]).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LNv1OutgoingPaymentSucceeded {
    contract_id: String,
    contract_amount: i64,
    gateway_key: String,
    payment_hash: String,
    timelock: i64,
    user_key: String,
    preimage: String,
}

impl<'de> Deserialize<'de> for LNv1OutgoingPaymentSucceeded {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        let contract_id = value["contract_id"]
            .as_str()
            .expect("Should be present")
            .to_string();
        let contract_amount = value["outgoing_contract"]["amount"]
            .as_i64()
            .expect("contract amount should be present");
        let gateway_key = value["outgoing_contract"]["contract"]["gateway_key"]
            .as_str()
            .expect("Should be present")
            .to_string();
        let payment_hash = value["outgoing_contract"]["contract"]["hash"]
            .as_str()
            .expect("Should be present")
            .to_string();
        let timelock = value["outgoing_contract"]["contract"]["timelock"]
            .as_i64()
            .expect("Should be present");
        let user_key = value["outgoing_contract"]["contract"]["user_key"]
            .as_str()
            .expect("Should be present")
            .to_string();
        let preimage = value["preimage"]
            .as_str()
            .expect("Should be present")
            .to_string();

        Ok(LNv1OutgoingPaymentSucceeded {
            contract_id,
            contract_amount,
            gateway_key,
            payment_hash,
            timelock,
            user_key,
            preimage,
        })
    }
}

impl LNv1OutgoingPaymentSucceeded {
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
        pg_client.execute("INSERT INTO lnv1_outgoing_payment_succeeded (log_id, ts, federation_id, federation_name, contract_id, contract_amount, gateway_key, payment_hash, timelock, user_key, preimage, gateway_epoch) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)", 
        &[&log_id, &timestamp, &federation_id.to_string(), &federation_name, &self.contract_id, &self.contract_amount, &self.gateway_key, &self.payment_hash, &self.timelock, &self.user_key, &self.preimage, &gateway_epoch]).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LNv2OutgoingPaymentSucceeded {
    payment_image: LNv2PaymentImage,
    target_federation: Option<String>,
}

impl<'de> Deserialize<'de> for LNv2OutgoingPaymentSucceeded {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let payment_image: LNv2PaymentImage =
            serde_json::from_value(value["payment_image"].clone())
                .map_err(|e| de::Error::custom(e.to_string()))?;
        let target_federation = value
            .get("target_federation")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        Ok(Self {
            payment_image,
            target_federation,
        })
    }
}

impl LNv2OutgoingPaymentSucceeded {
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
        pg_client.execute("INSERT INTO lnv2_outgoing_payment_succeeded (log_id, ts, federation_id, federation_name, gateway_epoch, payment_image, target_federation) VALUES ($1, $2, $3, $4, $5, $6, $7)", 
        &[&log_id, &timestamp, &federation_id.to_string(), &federation_name, &gateway_epoch, &self.payment_image.hash, &self.target_federation]).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LNv1OutgoingPaymentFailed {
    contract_id: String,
    contract_amount: i64,
    gateway_key: String,
    payment_hash: String,
    timelock: i64,
    user_key: String,
    error_reason: Option<String>,
}

impl<'de> Deserialize<'de> for LNv1OutgoingPaymentFailed {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        let contract_id = value["contract_id"]
            .as_str()
            .expect("Should be present")
            .to_string();
        let contract_amount = value["outgoing_contract"]["amount"]
            .as_i64()
            .expect("contract amount should be present");
        let gateway_key = value["outgoing_contract"]["contract"]["gateway_key"]
            .as_str()
            .expect("Should be present")
            .to_string();
        let payment_hash = value["outgoing_contract"]["contract"]["hash"]
            .as_str()
            .expect("Should be present")
            .to_string();
        let timelock = value["outgoing_contract"]["contract"]["timelock"]
            .as_i64()
            .expect("Should be present");
        let user_key = value["outgoing_contract"]["contract"]["user_key"]
            .as_str()
            .expect("Should be present")
            .to_string();
        let error_reason = LNv1OutgoingPaymentFailed::extract_error_reason(value)
            .expect("Could not get error_reason");

        Ok(LNv1OutgoingPaymentFailed {
            contract_id,
            contract_amount,
            gateway_key,
            payment_hash,
            timelock,
            user_key,
            error_reason,
        })
    }
}

impl LNv1OutgoingPaymentFailed {
    fn extract_error_reason(data: Value) -> anyhow::Result<Option<String>> {
        // Check for the 'error_type' key and handle different types of errors
        if let Some(error) = data.get("error") {
            if let Some(error_type) = error.get("error_type") {
                if let Some(lightning_error) = error_type.get("LightningPayError") {
                    if let Some(failed_payment) = lightning_error.get("lightning_error") {
                        if let Some(failure_reason) = failed_payment
                            .get("FailedPayment")
                            .and_then(|e| e.get("failure_reason"))
                        {
                            return Ok(Some(
                                failure_reason.as_str().unwrap_or_default().to_string(),
                            ));
                        }
                    }
                } else if let Some(invalid_outgoing_contract) =
                    error_type.get("InvalidOutgoingContract")
                {
                    if let Some(invoice_expired) = invalid_outgoing_contract
                        .get("error")
                        .and_then(|e| e.get("InvoiceExpired"))
                    {
                        return Ok(Some(format!(
                            "Invoice expired: {}",
                            invoice_expired.as_i64().unwrap_or_default()
                        )));
                    }
                }
            }
        }

        // Return None if no error reason is found
        Ok(None)
    }
}

impl LNv1OutgoingPaymentFailed {
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
        pg_client.execute("INSERT INTO lnv1_outgoing_payment_failed (log_id, ts, federation_id, federation_name, contract_id, contract_amount, gateway_key, payment_hash, timelock, user_key, error_reason, gateway_epoch) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)", 
    &[&log_id, &timestamp, &federation_id.to_string(), &federation_name, &self.contract_id, &self.contract_amount, &self.gateway_key, &self.payment_hash, &self.timelock, &self.user_key, &self.error_reason, &gateway_epoch]).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LNv2OutgoingPaymentFailed {
    payment_image: LNv2PaymentImage,
    error: String,
}

impl<'de> Deserialize<'de> for LNv2OutgoingPaymentFailed {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let payment_image: LNv2PaymentImage =
            serde_json::from_value(value["payment_image"].clone())
                .map_err(|e| de::Error::custom(e.to_string()))?;
        let error = value["error"]
            .as_str()
            .ok_or_else(|| de::Error::missing_field("error"))?
            .to_string();

        Ok(Self {
            payment_image,
            error,
        })
    }
}

impl LNv2OutgoingPaymentFailed {
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
        pg_client.execute("INSERT INTO lnv2_outgoing_payment_failed (log_id, ts, federation_id, federation_name, gateway_epoch, payment_image, error) VALUES ($1, $2, $3, $4, $5, $6, $7)", 
    &[&log_id, &timestamp, &federation_id.to_string(), &federation_name, &gateway_epoch, &self.payment_image.hash, &self.error]).await?;
        Ok(())
    }
}
