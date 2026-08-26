--migrate:up

CREATE TABLE transactions_new(
    id TEXT PRIMARY KEY,
    -- The sending account
    sender_id TEXT NULL REFERENCES accounts(id),
    -- The receiving account
    receiver_id TEXT NULL REFERENCES accounts(id),
    category_id TEXT NULL REFERENCES categories(id),
    transaction_date TEXT NOT NULL,
    amount INT NOT NULL DEFAULT 0,

    -- Prevent self transfers
    CHECK (receiver_id IS DISTINCT FROM sender_id)
    -- At least one column should be not null
    CHECK (receiver_id IS NOT NULL OR sender_id IS NOT NULL)
);

-- The app has not been released, so right now all former transactions are treated as expenses
INSERT INTO transactions_new(id,sender_id,category_id,transaction_date,amount)
SELECT id,account_id,category_id,transaction_date,amount
FROM transactions;

DROP TABLE transactions;
ALTER TABLE transactions_new RENAME TO transactions;

--migrate:down

CREATE TABLE transactions_new(
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES accounts(id),
    category_id TEXT NULL REFERENCES categories(id),
    transaction_date TEXT NOT NULL,
    amount INT NOT NULL DEFAULT 0
);

INSERT INTO transactions_new(id,account_id,category_id,transaction_date,amount)
SELECT id,sender_id,category_id,transaction_date,amount
FROM transactions;

DROP TABLE transactions;
ALTER TABLE transactions_new RENAME TO transactions;
