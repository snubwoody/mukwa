use crate::service::{Service, UpdateTransactionOpts};
use crate::{Money, ui};
use jiff::civil::Date;
use slint::{Model, SharedString, VecModel};
use std::rc::Rc;
use std::str::FromStr;
use tracing::info;
use uuid::Uuid;

// TODO: set the new transaction to editing
#[derive(Clone)]
pub struct AppState {
    service: Service,
    accounts: Rc<VecModel<ui::Account>>,
    // We can't map arrays in slint so we have to maintain duplicate arrays for comboboxes
    // see <https://github.com/slint-ui/slint/issues/1328>
    account_options: Rc<VecModel<(SharedString, SharedString)>>,
    transactions: Rc<VecModel<ui::Transaction>>,
}

impl AppState {
    pub fn new(service: Service) -> crate::Result<AppState> {
        let transactions_list: Vec<ui::Transaction> = service
            .fetch_transactions()?
            .iter()
            .map(|t| t.into())
            .collect();

        let transactions_model = Rc::new(VecModel::from(transactions_list));
        let account_list: Vec<ui::Account> =
            service.fetch_accounts()?.iter().map(|a| a.into()).collect();
        let account_options: Vec<_> = account_list
            .iter()
            .map(|a| (a.name.clone(), a.id.clone()))
            .collect();
        let accounts_model = Rc::new(VecModel::from(account_list));
        let account_options_model = Rc::new(VecModel::from(account_options));

        Ok(AppState {
            service,
            accounts: accounts_model,
            account_options: account_options_model,
            transactions: transactions_model,
        })
    }

    pub fn transactions(&self) -> Rc<VecModel<ui::Transaction>> {
        self.transactions.clone()
    }

    pub fn accounts(&self) -> Rc<VecModel<ui::Account>> {
        self.accounts.clone()
    }

    pub fn account_options(&self) -> Rc<VecModel<(SharedString, SharedString)>> {
        self.account_options.clone()
    }

    /// Creates a new account.
    pub fn create_account(&mut self, name: &str) -> crate::Result<()> {
        let account = self.service.create_account(name)?;
        info!(id=?account.id,"Created new account");
        self.accounts.push(account.clone().into());
        self.account_options.push((
            SharedString::from(account.id.to_string()),
            account.name.into(),
        ));
        Ok(())
    }

    pub fn create_transaction(&mut self) -> crate::Result<()> {
        let transaction = self.service.create_transaction(Default::default())?;
        info!(id=?transaction.id,"Created new transaction");
        self.transactions.push(transaction.into());
        Ok(())
    }

    pub fn delete_transaction(&mut self, id: &str) -> crate::Result<()> {
        let tid = Uuid::parse_str(id)?;
        self.service.delete_transaction(tid)?;
        info!("Deleted transaction {id}");
        let transactions = self
            .transactions
            .iter()
            .filter(|t| t.id.as_str() != id)
            .collect::<Vec<_>>();
        self.transactions.set_vec(transactions);
        Ok(())
    }

    pub fn duplicate_transaction(&mut self, id: &str) -> crate::Result<()> {
        let tid = Uuid::parse_str(id)?;
        let transaction = self.service.duplicate_transaction(tid)?;
        self.transactions.push(transaction.into());
        info!("Duplicated transaction {id}");
        Ok(())
    }

    pub fn update_transaction(
        &mut self,
        id: &str,
        account_id: &str,
        outflow: &str,
        inflow: &str,
        date: &str,
    ) -> crate::Result<()> {
        let account_id = Uuid::parse_str(account_id).ok();
        let outflow = Money::from_str(outflow).ok();
        let inflow = Money::from_str(inflow).ok();
        let date = Date::strptime("%Y-%m-%d", date).ok();

        let mut sender_id = None;
        let mut receiver_id = None;
        let mut amount = None;

        if let Some(value) = outflow {
            amount = Some(value);
            sender_id = account_id;
        }

        if let Some(value) = inflow {
            amount = Some(value);
            receiver_id = account_id;
        }

        let opts = UpdateTransactionOpts {
            id: Uuid::parse_str(id)?,
            date,
            sender_id,
            amount,
            category_id: None,
            receiver_id,
        };

        self.service.update_transaction(opts)?;
        info!(id=?id,"Updated transaction");
        self.reset_transactions()?;
        Ok(())
    }

    fn reset_transactions(&mut self) -> crate::Result<()> {
        let transactions: Vec<ui::Transaction> = self
            .service
            .fetch_transactions()?
            .iter()
            .map(|t| t.into())
            .collect();
        self.transactions.set_vec(transactions);
        Ok(())
    }

    pub fn get_account(&self, id: SharedString) -> Option<ui::Account> {
        self.accounts.iter().find(|a| a.id == id)
    }
}
