// Mukwa - Personal finance
// Copyright (C) 2026  Wakunguma Kalimukwa
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

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
            let formatter = CurrencyFormatter::new().unwrap();
            formatter
        })
        .bench_refs(|formatter| {
            formatter
                .format_currency(divan::black_box(Money::new(N)))
                .unwrap();
        });
}
