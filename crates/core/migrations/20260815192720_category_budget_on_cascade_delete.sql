--migrate:up
create table budgets_new(
    id TEXT primary key,
    category_id TEXT not null references categories(id) on delete cascade,
    month int not null,
    year int not null,
    amount int default 0,

    -- Only one budget per month
    unique(category_id,month,year),

    check (month >= 0 and year >= 0 and amount >= 0)
);

insert into budgets_new(id,category_id,month,year,amount)
select id,category_id,month,year,amount from budgets;

drop table budgets;

alter table budgets_new
rename to budgets;

--migrate:down
create table budgets_new(
    id TEXT primary key,
    category_id TEXT not null references categories(id),
    month int not null,
    year int not null,
    amount int default 0,

    -- Only one budget per month
    unique(category_id,month,year),

    check (month >= 0 and year >= 0 and amount >= 0)
);


insert into budgets_new(id,category_id,month,year,amount)
select id,category_id,month,year,amount from budgets;

drop table budgets;

alter table budgets_new
rename to budgets;


