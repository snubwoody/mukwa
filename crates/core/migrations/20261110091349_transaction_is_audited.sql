--migrate:up
ALTER TABLE transactions
ADD COLUMN is_audited BOOL NOT NULL DEFAULT 0;

--migrate:down
ALTER TABLE transactions
DROP COLUMN is_audited;
