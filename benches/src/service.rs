// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use divan::Bencher;
use mukwa_core::migrator::Migrator;
use mukwa_core::service::{AccountType, Service};
use rusqlite::Connection;
use tempfile::tempdir;

fn main() {
    divan::main();
}

#[divan::bench(consts = [1,10,50,100])]
fn create_account<const N: usize>(bencher: Bencher) {
    bencher
        .with_inputs(move || {
            let temp = tempdir().unwrap();
            let path = temp.path().join("data.sqlite");
            let mut connection = Connection::open(path).unwrap();
            let mut migrator = Migrator::new();
            migrator.load_embedded().unwrap();
            migrator.migrate(&mut connection).unwrap();
            let service = Service::new(connection);
            (service, temp)
        })
        .bench_refs(|(service, _temp)| {
            for _ in 0..N {
                service
                    .create_account("My account", AccountType::Cash)
                    .unwrap();
            }
        });
}
