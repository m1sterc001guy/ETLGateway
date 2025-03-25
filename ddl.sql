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
