// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use mukwa_core::Money;
use mukwa_core::migrator::Migrator;
use mukwa_core::service::{AccountType, Service};
use rusqlite::Connection;
use std::hint::black_box;
use tempfile::tempdir;

fn search(c: &mut Criterion) {
    let mut group = c.benchmark_group("search");
    for n in [1, 10, 50] {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, n| {
            b.iter_batched(
                || {
                    let temp = tempdir().unwrap();
                    let path = temp.path().join("data.sqlite");
                    let mut connection = Connection::open(path).unwrap();
                    let mut migrator = Migrator::new();
                    migrator.load_embedded().unwrap();
                    migrator.migrate(&mut connection).unwrap();
                    let service = Service::new(connection);
                    let account = service
                        .create_account("My account", AccountType::Cash)
                        .unwrap();
                    for _ in 0..*n {
                        service
                            .create_expense()
                            .amount(Money::new(10))
                            .account(account.id)
                            .submit()
                            .unwrap();
                    }

                    let app = mukwa::ui::MainWindow::new().unwrap();
                    let controller =
                        mukwa::controllers::TransactionController::bind(&app, &service).unwrap();

                    (controller, temp)
                },
                |(controller, _temp)| {
                    black_box(controller.search("My account").unwrap());
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

criterion_group!(benches, search);
criterion_main!(benches);
