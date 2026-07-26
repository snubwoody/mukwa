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
use mukwa::migrator::Migrator;
use mukwa::service::Service;
use rusqlite::Connection;
use tempfile::tempdir;

fn main() {
    divan::main();
}

// TODO: measure thread contention: https://nikolaivazquez.com/blog/divan/#measure-thread-contention
#[divan::bench(consts = [1,10,50,100])]
fn create_account<const N: usize>(bencher: Bencher) {
    bencher
        .with_inputs(move || {
            let temp = tempdir().unwrap();
            let path = temp.path().join("data.sqlite");
            let mut connection = Connection::open(path).unwrap();
            let mut migrator = Migrator::new();
            migrator.load_from_dir("./migrations").unwrap();
            migrator.migrate(&mut connection).unwrap();
            let service = Service::new(connection);
            (service, temp)
        })
        .bench_refs(|(service, _temp)| {
            for _ in 0..N {
                service.create_account("My account").unwrap();
            }
        });
}
