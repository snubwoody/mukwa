use divan::Bencher;
use mukwa::service::{Service, Transaction, UpdateTransactionOpts};
use mukwa::Money;
use tempfile::tempdir;

fn main() {
    divan::main();
}

#[divan::bench(consts = [1,10,50,100])]
fn update_transaction<const N: usize>(bencher: Bencher) {
    bencher
        .with_inputs(move || {
            let temp = tempdir().unwrap();
            let path = temp.path().join("app.data");
            let mut service = Service::open("temp.data");
            for _ in 0..N {
                service.create_transaction().unwrap();
            }
            let transaction = service.create_transaction().unwrap();
            let update_opts = UpdateTransactionOpts {
                id: transaction.id,
                account_id: None,
                amount: Some(Money::new(200)),
                date: None,
            };
            (service, update_opts)
        })
        .bench_refs(|(service, opts)| service.update_transaction(opts.clone()));
}
