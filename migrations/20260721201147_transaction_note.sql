--migrate:up

ALTER TABLE transactions
ADD COLUMN note TEXT NULL;

--migrate:down

ALTER TABLE transactions
DROP COLUMN note;
