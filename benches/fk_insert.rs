// Copyright 2025 Oxibase Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use criterion::{criterion_group, criterion_main, Criterion};
use oxibase::Database;
use std::hint::black_box;

fn fk_insert_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_performance");

    group.bench_function("insert_no_fk", |b| {
        let db = Database::open_in_memory().unwrap();
        db.execute(
            "CREATE TABLE orders (id INTEGER PRIMARY KEY, user_id INTEGER)",
            (),
        )
        .unwrap();

        let mut i = 0;
        b.iter(|| {
            i += 1;
            let sql = format!("INSERT INTO orders (id, user_id) VALUES ({}, 1)", i);
            db.execute(black_box(&sql), ()).unwrap()
        });
    });

    group.bench_function("insert_with_fk", |b| {
        let db = Database::open_in_memory().unwrap();
        db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY)", ())
            .unwrap();
        db.execute("INSERT INTO users (id) VALUES (1)", ()).unwrap();

        db.execute("CREATE TABLE orders (id INTEGER PRIMARY KEY, user_id INTEGER, FOREIGN KEY (user_id) REFERENCES users(id))", ()).unwrap();

        let mut i = 0;
        b.iter(|| {
            i += 1;
            let sql = format!("INSERT INTO orders (id, user_id) VALUES ({}, 1)", i);
            db.execute(black_box(&sql), ()).unwrap()
        });
    });

    group.finish();
}

criterion_group!(benches, fk_insert_benchmark);
criterion_main!(benches);
