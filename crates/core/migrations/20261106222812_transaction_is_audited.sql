--migrate:up
ALTER TABLE transactions
ADD COLUMN is_audited BOOL DEFAULT 0;

--migrate:down
ALTER TABLE transactions
DROP COLUMN is_audited;