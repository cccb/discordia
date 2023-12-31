
CREATE TABLE members (
    id                INTEGER           PRIMARY KEY AUTOINCREMENT,
    name              VARCHAR(100)      NOT NULL,
    email             VARCHAR(100)      NOT NULL,
    notes             TEXT              NOT NULL,
    membership_start  TEXT              NOT NULL, -- DATE
    membership_end    TEXT              NULL     DEFAULT NULL,
    fee               DECIMAL(10, 2)    NOT NULL,
    interval          INTEGER           NOT NULL DEFAULT 1,
    last_payment_at   TEXT              NOT NULL, -- DATE
    last_bank_transaction_at TEXT       NOT NULL, -- DATE
    last_bank_transaction_number INTEGER  NOT NULL,
    account_calculated_at TEXT          NOT NULL, -- DATE
    account           DECIMAL(10, 2)    NOT NULL DEFAULT '0.00'
);


CREATE TABLE bank_import_member_ibans (
    member_id         INTEGER           NOT NULL,
    iban              VARCHAR(100)      NOT NULL,

    match_subject     VARCHAR(255)      NULL,

    split_amount      DECIMAL(10, 2)    NULL,

    FOREIGN KEY (member_id) REFERENCES members(id)
      ON DELETE CASCADE,

    PRIMARY KEY (member_id, iban)
);


CREATE TABLE transactions (
    id                INTEGER           PRIMARY KEY AUTOINCREMENT,
    member_id         INTEGER           NOT NULL,
    date              TEXT              NOT NULL, -- DATE
    account_name      VARCHAR(100)      NOT NULL,
    amount            DECIMAL(10, 2)    NOT NULL,
    description       TEXT              NOT NULL,

    FOREIGN KEY (member_id) REFERENCES members(id)
      ON DELETE CASCADE
);

