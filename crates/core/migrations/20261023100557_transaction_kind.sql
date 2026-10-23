--migrate:up
CREATE TABLE transaction_kind(
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL
);

INSERT INTO transaction_kind(id,title)
VALUES
    (0,'Regular'),
    (1,'Starting balance');

ALTER TABLE transactions
ADD COLUMN kind INTEGER NOT NULL REFERENCES transaction_kind(id) DEFAULT 0;

--migrate:down
ALTER TABLE transactions
DROP COLUMN kind;

DROP TABLE transaction_kind;
