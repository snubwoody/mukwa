use jiff::civil::date;
use mukwa::service::{CreateTransactionOpts, Service, UpdateTransactionOpts};
use mukwa::{Money, create_test_db};

#[test]
fn create_account() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    let account = service.create_account("My account")?;
    assert_eq!(account.name, "My account");

    let connection = service.connection();
    let name = connection.query_one(
        "SELECT name FROM accounts WHERE id=?",
        [account.id.to_string()],
        |row| row.get::<_, String>(0),
    )?;
    assert_eq!(name, account.name);
    Ok(())
}

#[test]
fn create_transaction_fails_with_no_accounts() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    let result = service.create_transaction(Default::default());
    assert!(result.is_err());
    Ok(())
}

#[test]
fn create_transaction_selects_first_account() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    let account = service.create_account("")?;
    let transaction = service.create_transaction(Default::default())?;
    assert_eq!(transaction.account_id, account.id);
    Ok(())
}

#[test]
fn create_transaction() -> mukwa::Result<()> {
    let connection = create_test_db();

    let service = Service::new(connection);
    let account = service.create_account("")?;

    let opts = CreateTransactionOpts {
        amount: Money::new(200),
        date: date(2020, 10, 20),
        ..Default::default()
    };
    let transaction = service.create_transaction(opts)?;
    assert_eq!(transaction.account_id, account.id);
    assert_eq!(transaction.category_id, None);
    assert_eq!(transaction.amount, Money::new(200));
    assert_eq!(transaction.date, date(2020, 10, 20));

    service
        .connection()
        .query_one("SELECT * FROM transactions", [], |row| {
            let amount: i64 = row.get("amount")?;
            let date: String = row.get("transaction_date")?;
            assert_eq!(amount, Money::new(200).inner());
            assert_eq!(date, "2020-10-20");
            Ok(())
        })?;

    Ok(())
}

#[test]
fn update_transaction() -> mukwa::Result<()> {
    let connection = create_test_db();

    let service = Service::new(connection);
    let account = service.create_account("")?;
    let account2 = service.create_account("")?;

    let create_opts = CreateTransactionOpts {
        account_id: Some(account.id),
        ..Default::default()
    };

    let transaction = service.create_transaction(create_opts)?;
    let update_opts = UpdateTransactionOpts {
        id: transaction.id,
        account_id: Some(account2.id),
        amount: Some(Money::new(500)),
        date: Some(date(1990, 1, 1)),
        ..Default::default()
    };

    service.update_transaction(update_opts)?;

    service
        .connection()
        .query_one("SELECT * FROM transactions", [], |row| {
            let amount: i64 = row.get("amount")?;
            let date: String = row.get("transaction_date")?;
            let account_id: String = row.get("account_id")?;
            assert_eq!(amount, Money::new(500).inner());
            assert_eq!(date, "1990-01-01");
            assert_eq!(account_id, account2.id.to_string());
            Ok(())
        })?;

    Ok(())
}

#[test]
fn fetch_accounts() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    let a1 = service.create_account("My account")?;
    let a2 = service.create_account("My account")?;

    let accounts = service.fetch_accounts()?;
    assert_eq!(accounts.len(), 2);
    assert!(accounts.contains(&a1));
    assert!(accounts.contains(&a2));
    Ok(())
}

#[test]
fn fetch_transactions() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    service.create_account("My account")?;
    let t1 = service.create_transaction(Default::default())?;
    let t2 = service.create_transaction(Default::default())?;
    let t3 = service.create_transaction(Default::default())?;

    let transactions = service.fetch_transactions()?;
    assert_eq!(transactions.len(), 3);
    assert!(transactions.contains(&t1));
    assert!(transactions.contains(&t2));
    assert!(transactions.contains(&t3));
    Ok(())
}

#[test]
fn fetch_empty_accounts() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    let accounts = service.fetch_accounts()?;
    assert!(accounts.is_empty());
    Ok(())
}
