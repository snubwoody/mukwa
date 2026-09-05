--migrate:up

ALTER TABLE category_groups
ADD COLUMN is_meta INT NOT NULL DEFAULT FALSE;

INSERT INTO category_groups(id,title,is_meta)
VALUES ('01a0727e-db55-7e4c-83fc-9eb839c2c190','Credit payments',TRUE);

CREATE TABLE categories_new(
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    group_id TEXT NOT NULL REFERENCES category_groups(id) ON DELETE CASCADE,
    account_id TEXT NULL REFERENCES accounts(id) ON DELETE CASCADE
);

INSERT INTO categories_new(id,title,group_id)
SELECT id,title,group_id FROM categories;

DROP TABLE categories;

ALTER TABLE categories_new
RENAME TO categories;

--migrate:down

DELETE FROM category_groups
WHERE id = '01a0727e-db55-7e4c-83fc-9eb839c2c190';

ALTER TABLE category_groups
DROP COLUMN is_meta;

CREATE TABLE categories_new(
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    group_id TEXT NOT NULL REFERENCES category_groups(id) ON DELETE CASCADE
);

INSERT INTO categories_new(id,title,group_id)
SELECT id,title,group_id FROM categories;

DROP TABLE categories;

ALTER TABLE categories_new
RENAME TO categories;
