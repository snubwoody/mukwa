--migrate:up

CREATE TABLE unconfirmed_transactions(
    id INTEGER PRIMARY KEY,
    transaction_id TEXT NOT NULL UNIQUE REFERENCES transactions(id) ON DELETE CASCADE
);

--migrate:down

DROP TABLE unconfirmed_transactions;
