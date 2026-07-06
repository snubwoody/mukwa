--migrate:up

CREATE TABLE accounts(
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE categories(
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE transactions(
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES accounts(id),
    category_id TEXT NULL REFERENCES categories(id),
    transaction_date TEXT NOT NULL,
    amount INT NOT NULL DEFAULT 0
);

--migrate:down

DROP TABLE transactions;
DROP TABLE categories;
DROP TABLE accounts;
