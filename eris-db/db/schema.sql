
CREATE TABLE members (
    id                INTEGER           PRIMARY KEY AUTOINCREMENT,
    name              VARCHAR(100)      NOT NULL,
    email             VARCHAR(100)      NOT NULL,
    notes             TEXT              NOT NULL,
    membership_start  TEXT              NOT NULL, -- DATE
    membership_end    TEXT              NULL     DEFAULT NULL,
    fee               DECIMAL(10, 2)    NOT NULL,
    interval          INTEGER           NOT NULL DEFAULT 1,
    last_payment      TEXT              NOT NULL, -- DATE
    account           DECIMAL(10, 2)    NOT NULL DEFAULT '0.00'
);


CREATE TABLE bank_import_member_ibans (
    member_id         INTEGER           NOT NULL,
    iban_hash         VARCHAR(100)      NOT NULL,

    match_subject     VARCHAR(255)      NULL,

    split_amount      DECIMAL(10, 2)    NULL,

    FOREIGN KEY (member_id) REFERENCES members(id)
      ON DELETE CASCADE,

    PRIMARY KEY (member_id, iban_hash)
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

CREATE TABLE state (
    accounts_calculated_at  TEXT -- DATE
);

INSERT INTO state ( accounts_calculated_at ) VALUES (date());
