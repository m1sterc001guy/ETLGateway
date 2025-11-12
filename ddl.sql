CREATE TABLE lnv1_outgoing_payment_started (
    log_id BIGINT PRIMARY KEY,
	ts TIMESTAMP NOT NULL,
	federation_id TEXT NOT NULL,
	federation_name TEXT NOT NULL,
	contract_id TEXT NOT NULL,
	invoice_amount BIGINT NOT NULL,
	operation_id TEXT NOT NULL
);

CREATE TABLE lnv1_outgoing_payment_succeeded (
    log_id BIGINT PRIMARY KEY,
	ts TIMESTAMP NOT NULL,
	federation_id TEXT  NOT NULL,
	federation_name TEXT NOT NULL,
	contract_id TEXT NOT NULL,
	contract_amount BIGINT NOT NULL,
	gateway_key TEXT NOT NULL,
	payment_hash TEXT NOT NULL,
	timelock BIGINT NOT NULL,
	user_key TEXT NOT NULL,
	preimage TEXT NOT NULL
);

CREATE TABLE lnv1_outgoing_payment_failed (
    log_id BIGINT PRIMARY KEY,
	ts TIMESTAMP NOT NULL,
	federation_id TEXT  NOT NULL,
	federation_name TEXT NOT NULL,
	contract_id TEXT NOT NULL,
	contract_amount BIGINT NOT NULL,
	gateway_key TEXT NOT NULL,
	payment_hash TEXT NOT NULL,
	timelock BIGINT NOT NULL,
	user_key TEXT NOT NULL,
	error_reason TEXT
);

CREATE TABLE lnv1_incoming_payment_started (
    log_id BIGINT PRIMARY KEY,
	ts TIMESTAMP NOT NULL,
	federation_id TEXT NOT NULL,
	federation_name TEXT NOT NULL,
	contract_id TEXT NOT NULL,
	contract_amount BIGINT NOT NULL,
	invoice_amount BIGINT NOT NULL,
	operation_id TEXT NOT NULL,
	payment_hash TEXT NOT NULL
);

CREATE TABLE lnv1_incoming_payment_succeeded (
    log_id BIGINT PRIMARY KEY,
	ts TIMESTAMP NOT NULL,
	federation_id TEXT NOT NULL,
	federation_name TEXT NOT NULL,
	payment_hash TEXT NOT NULL,
	preimage TEXT NOT NULL
);

CREATE TABLE lnv1_incoming_payment_failed (
    log_id BIGINT PRIMARY KEY,
	ts TIMESTAMP NOT NULL,
	federation_id TEXT NOT NULL,
	federation_name TEXT NOT NULL,
	payment_hash TEXT NOT NULL,
	error_reason TEXT NOT NULL
);

CREATE TABLE lnv1_complete_lightning_payment_succeeded (
    log_id BIGINT PRIMARY KEY,
	ts TIMESTAMP NOT NULL,
	federation_id TEXT NOT NULL,
	federation_name TEXT NOT NULL,
	payment_hash TEXT NOT NULL
);

ALTER TABLE lnv1_outgoing_payment_started ADD COLUMN gateway_epoch INT NOT NULL DEFAULT 0;
ALTER TABLE lnv1_outgoing_payment_succeeded ADD COLUMN gateway_epoch INT NOT NULL DEFAULT 0;
ALTER TABLE lnv1_outgoing_payment_failed ADD COLUMN gateway_epoch INT NOT NULL DEFAULT 0;
ALTER TABLE lnv1_incoming_payment_started ADD COLUMN gateway_epoch INT NOT NULL DEFAULT 0;
ALTER TABLE lnv1_incoming_payment_succeeded ADD COLUMN gateway_epoch INT NOT NULL DEFAULT 0;
ALTER TABLE lnv1_incoming_payment_failed ADD COLUMN gateway_epoch INT NOT NULL DEFAULT 0;
ALTER TABLE lnv1_complete_lightning_payment_succeeded ADD COLUMN gateway_epoch INT NOT NULL DEFAULT 0;

ALTER TABLE lnv1_outgoing_payment_started DROP CONSTRAINT lnv1_outgoing_payment_started_pkey;
ALTER TABLE lnv1_outgoing_payment_started ADD PRIMARY KEY (log_id, gateway_epoch);

ALTER TABLE lnv1_outgoing_payment_succeeded DROP CONSTRAINT lnv1_outgoing_payment_succeeded_pkey;
ALTER TABLE lnv1_outgoing_payment_succeeded ADD PRIMARY KEY (log_id, gateway_epoch);

ALTER TABLE lnv1_outgoing_payment_failed DROP CONSTRAINT lnv1_outgoing_payment_failed_pkey;
ALTER TABLE lnv1_outgoing_payment_failed ADD PRIMARY KEY (log_id, gateway_epoch);

ALTER TABLE lnv1_incoming_payment_started DROP CONSTRAINT lnv1_incoming_payment_started_pkey;
ALTER TABLE lnv1_incoming_payment_started ADD PRIMARY KEY (log_id, gateway_epoch);

ALTER TABLE lnv1_incoming_payment_succeeded DROP CONSTRAINT lnv1_incoming_payment_succeeded_pkey;
ALTER TABLE lnv1_incoming_payment_succeeded ADD PRIMARY KEY (log_id, gateway_epoch);

ALTER TABLE lnv1_incoming_payment_failed DROP CONSTRAINT lnv1_incoming_payment_failed_pkey;
ALTER TABLE lnv1_incoming_payment_failed ADD PRIMARY KEY (log_id, gateway_epoch);

ALTER TABLE lnv1_complete_lightning_payment_succeeded DROP CONSTRAINT lnv1_complete_lightning_payment_succeeded_pkey;
ALTER TABLE lnv1_complete_lightning_payment_succeeded ADD PRIMARY KEY (log_id, gateway_epoch);

CREATE TABLE lnv2_outgoing_payment_started(
	log_id BIGINT NOT NULL,
	ts TIMESTAMP NOT NULL,
	federation_id TEXT NOT NULL,
	federation_name TEXT NOT NULL,
	gateway_epoch INT NOT NULL,
	invoice_amount BIGINT NOT NULL,
	max_delay BIGINT NOT NULL,
	min_contract_amount BIGINT NOT NULL,
	operation_start TIMESTAMP NOT NULL,
	amount BIGINT NOT NULL,
	claim_pk TEXT NOT NULL,
	ephemeral_pk TEXT NOT NULL,
	expiration BIGINT NOT NULL,
	payment_image TEXT NOT NULL,
	refund_pk TEXT NOT NULL,
	PRIMARY KEY (log_id, gateway_epoch)
);

CREATE TABLE lnv2_outgoing_payment_succeeded(
	log_id BIGINT NOT NULL,
	ts TIMESTAMP NOT NULL,
	federation_id TEXT NOT NULL,
	federation_name TEXT NOT NULL,
	gateway_epoch INT NOT NULL,
	payment_image TEXT NOT NULL,
	target_federation TEXT,
	PRIMARY KEY (log_id, gateway_epoch)
);

CREATE TABLE lnv2_outgoing_payment_failed(
	log_id BIGINT NOT NULL,
	ts TIMESTAMP NOT NULL,
	federation_id TEXT NOT NULL,
	federation_name TEXT NOT NULL,
	gateway_epoch INT NOT NULL,
	payment_image TEXT NOT NULL,
	error TEXT NOT NULL,
	PRIMARY KEY (log_id, gateway_epoch)
);

CREATE TABLE lnv2_incoming_payment_started(
	log_id BIGINT NOT NULL,
	ts TIMESTAMP NOT NULL,
	federation_id TEXT NOT NULL,
	federation_name TEXT NOT NULL,
	gateway_epoch INT NOT NULL,
	amount BIGINT NOT NULL,
	claim_pk TEXT NOT NULL,
	ephemeral_pk TEXT NOT NULL,
	expiration BIGINT NOT NULL,
	payment_image TEXT NOT NULL,
	refund_pk TEXT NOT NULL,
	invoice_amount BIGINT NOT NULL,
	operation_start TIMESTAMP NOT NULL,	
	PRIMARY KEY (log_id, gateway_epoch)
);

CREATE TABLE lnv2_incoming_payment_succeeded(
	log_id BIGINT NOT NULL,
	ts TIMESTAMP NOT NULL,
	federation_id TEXT NOT NULL,
	federation_name TEXT NOT NULL,
	gateway_epoch INT NOT NULL,
	payment_image TEXT NOT NULL,
	PRIMARY KEY (log_id, gateway_epoch)
);

CREATE TABLE lnv2_incoming_payment_failed(
	log_id BIGINT NOT NULL,
	ts TIMESTAMP NOT NULL,
	federation_id TEXT NOT NULL,
	federation_name TEXT NOT NULL,
	gateway_epoch INT NOT NULL,
	payment_image TEXT NOT NULL,
	error TEXT NOT NULL,
	PRIMARY KEY (log_id, gateway_epoch)
);

CREATE TABLE lnv2_complete_lightning_payment_succeeded(
	log_id BIGINT NOT NULL,
	ts TIMESTAMP NOT NULL,
	federation_id TEXT NOT NULL,
	federation_name TEXT NOT NULL,
	gateway_epoch INT NOT NULL,
	payment_image TEXT NOT NULL,
	PRIMARY KEY (log_id, gateway_epoch)
);


DROP TABLE lnv1_outgoing_payment_started;
DROP TABLE lnv1_outgoing_payment_succeeded;
DROP TABLE lnv1_outgoing_payment_failed;
DROP TABLE lnv1_incoming_payment_started;
DROP TABLE lnv1_incoming_payment_succeeded;
DROP TABLE lnv1_incoming_payment_failed;
DROP TABLE lnv1_complete_lightning_payment_succeeded;

SELECT * FROM lnv1_outgoing_payment_started;
SELECT * FROM lnv1_outgoing_payment_succeeded;
SELECT * FROM lnv1_outgoing_payment_failed;
SELECT * FROM lnv1_incoming_payment_started;
SELECT * FROM lnv1_incoming_payment_succeeded;
SELECT * FROM lnv1_incoming_payment_failed;
SELECT * FROM lnv1_complete_lightning_payment_succeeded;

TRUNCATE TABLE lnv1_outgoing_payment_started;
TRUNCATE TABLE lnv1_outgoing_payment_succeeded;
TRUNCATE TABLE lnv1_outgoing_payment_failed;
TRUNCATE TABLE lnv1_incoming_payment_started;
TRUNCATE TABLE lnv1_incoming_payment_succeeded;
TRUNCATE TABLE lnv1_incoming_payment_failed;
TRUNCATE TABLE lnv1_complete_lightning_payment_succeeded;
