// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use criterion::{BatchSize, BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use mukwa_core::Money;
use mukwa_core::fmt::CurrencyFormatter;

fn bench_format_money(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_money");

    for &n in &[1, 50, 100, 1000, 50_000] {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                CurrencyFormatter::new,
                |formatter| {
                    black_box(formatter.format_money(black_box(Money::new(n))).unwrap());
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

criterion_group!(benches, bench_format_money);
criterion_main!(benches);
