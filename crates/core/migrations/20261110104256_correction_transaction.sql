--migrate:up
INSERT INTO transaction_kind(id,title)
VALUES
    (2,'Correction');
--migrate:down
DELETE FROM transaction_kind WHERE id = 2;
