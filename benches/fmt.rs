// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use divan::Bencher;
use mukwa::Money;
use mukwa::fmt::CurrencyFormatter;

fn main() {
    divan::main();
}

#[divan::bench(consts = [1,50,100,1000,50_000])]
fn format_money<const N: i64>(bencher: Bencher) {
    bencher
        .with_inputs(|| {
            let formatter = CurrencyFormatter::new();
            formatter
        })
        .bench_refs(|formatter| {
            formatter
                .format_money(divan::black_box(Money::new(N)))
                .unwrap();
        });
}
