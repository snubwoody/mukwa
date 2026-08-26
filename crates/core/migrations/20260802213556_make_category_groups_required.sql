--migrate:up
CREATE TABLE categories_new(
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    group_id TEXT NOT NULL REFERENCES category_groups(id),
    deleted_at INT NULL
);

INSERT INTO categories_new(id,title,deleted_at,group_id)
SELECT id,title,deleted_at,group_id FROM categories;

DROP TABLE categories;

ALTER TABLE categories_new
RENAME TO categories;

--migrate:down

CREATE TABLE categories_new(
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    group_id TEXT NULL REFERENCES category_groups(id),
    deleted_at INT NULL
);

INSERT INTO categories_new(id,title,deleted_at,group_id)
SELECT id,title,deleted_at,group_id FROM categories;

DROP TABLE categories;

ALTER TABLE categories_new
RENAME TO categories;
