--migrate:up
CREATE TABLE category_groups(
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    deleted_at INT NULL
);

CREATE TABLE categories_new(
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    group_id TEXT NULL REFERENCES category_groups(id),
    deleted_at INT NULL
);

INSERT INTO categories_new(id,title,deleted_at)
SELECT id,title,deleted_at FROM categories;

DROP TABLE categories;

ALTER TABLE categories_new
RENAME TO categories;

--migrate:down

CREATE TABLE categories_new(
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    deleted_at INT NULL
);

INSERT INTO categories_new(id,title,deleted_at)
SELECT id,title,deleted_at FROM categories;

DROP TABLE categories;

ALTER TABLE categories_new
RENAME TO categories;

DROP TABLE category_groups;
