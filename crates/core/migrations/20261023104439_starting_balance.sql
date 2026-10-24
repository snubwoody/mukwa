--migrate:up
UPDATE transactions
SET kind = 1
WHERE note = 'Starting balance';

--migrate:down
UPDATE transactions
SET kind = 0
WHERE note = 'Starting balance';