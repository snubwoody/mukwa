--migrate:up

ALTER TABLE categories
RENAME COLUMN name TO title;

ALTER TABLE categories
ADD COLUMN deleted_at INT NULL;

CREATE TABLE budgets(
    id TEXT PRIMARY KEY,
    category_id TEXT NOT NULL REFERENCES categories(id),
    month INT NOT NULL,
    year INT NOT NULL,
    amount INT DEFAULT 0,

    -- Only one budget per month
    UNIQUE(month,year),

    CHECK (month >= 0 AND year >= 0 AND amount >= 0)
);

--migrate:down

DROP TABLE budgets;

ALTER TABLE categories
RENAME COLUMN title TO name;

ALTER TABLE categories
DROP COLUMN deleted_at;

