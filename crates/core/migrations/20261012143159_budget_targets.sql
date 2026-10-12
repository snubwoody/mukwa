--migrate:up
CREATE TABLE budget_targets(
    id INTEGER PRIMARY KEY,
    category_id TEXT NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    amount INTEGER NOT NULL DEFAULT 0,

    -- One target per budget
    UNIQUE(category_id),
    CHECK(amount >= 0)
);

--migrate:down
DROP TABLE budget_targets;
