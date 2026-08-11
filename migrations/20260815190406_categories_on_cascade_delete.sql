--migrate:up

CREATE TABLE transactions_new(
    id TEXT PRIMARY KEY,
    -- The sending account
    sender_id TEXT NULL REFERENCES accounts(id),
    -- The receiving account
    receiver_id TEXT NULL REFERENCES accounts(id),
    category_id TEXT NULL REFERENCES categories(id) ON DELETE SET NULL,
    transaction_date TEXT NOT NULL,
    note TEXT NULL,
    amount INT NOT NULL DEFAULT 0,

    -- Prevent self transfers
    CHECK (receiver_id IS DISTINCT FROM sender_id)
    -- At least one column should be not null
    CHECK (receiver_id IS NOT NULL OR sender_id IS NOT NULL)
);

INSERT INTO transactions_new(id,sender_id,receiver_id,category_id,transaction_date,amount,note)
SELECT id,sender_id,receiver_id,category_id,transaction_date,amount,note
FROM transactions;

DROP TABLE transactions;
ALTER TABLE transactions_new RENAME TO transactions;

--migrate:down

CREATE TABLE transactions_new(
    id TEXT PRIMARY KEY,
    -- The sending account
    sender_id TEXT NULL REFERENCES accounts(id),
    -- The receiving account
    receiver_id TEXT NULL REFERENCES accounts(id),
    category_id TEXT NULL REFERENCES categories(id) ON DELETE SET NULL,
    transaction_date TEXT NOT NULL,
    note TEXT NULL,
    amount INT NOT NULL DEFAULT 0,

    -- Prevent self transfers
    CHECK (receiver_id IS DISTINCT FROM sender_id)
    -- At least one column should be not null
    CHECK (receiver_id IS NOT NULL OR sender_id IS NOT NULL)
);

INSERT INTO transactions_new(id,sender_id,receiver_id,category_id,transaction_date,amount,note)
SELECT id,sender_id,receiver_id,category_id,transaction_date,amount,note
FROM transactions;

DROP TABLE transactions;
ALTER TABLE transactions_new RENAME TO transactions;