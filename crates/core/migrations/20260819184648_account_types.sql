--migrate:up

CREATE TABLE account_types(
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL
);

INSERT INTO account_types(id,title,description)
VALUES
    (1,'Cash','Cash accounts hold money that is readily available.'),
    (2,'Credit','Credit accounts let you borrow money to spend it.');

ALTER TABLE accounts
ADD COLUMN account_type_id INTEGER NOT NULL REFERENCES account_types(id) DEFAULT 1;

--migrate:down

ALTER TABLE accounts
DROP COLUMN account_type_id;

DROP TABLE account_types;