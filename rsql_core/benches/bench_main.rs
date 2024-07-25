use criterion::criterion_main;

mod benchmarks;

#[cfg(feature = "driver-libsql")]
criterion_main! {
    benchmarks::duckdb::all,
    benchmarks::libsql::all,
    benchmarks::postgres::all,
    benchmarks::postgresql::all,
    benchmarks::rusqlite::all,
    benchmarks::sqlite::all,
}

#[cfg(not(feature = "driver-libsql"))]
criterion_main! {
    benchmarks::duckdb::all,
    benchmarks::postgres::all,
    benchmarks::postgresql::all,
    benchmarks::rusqlite::all,
    benchmarks::sqlite::all,
}
