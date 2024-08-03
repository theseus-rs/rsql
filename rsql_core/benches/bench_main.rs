use criterion::criterion_main;

mod benchmarks;

criterion_main! {
    benchmarks::postgres::all,
    benchmarks::postgresql::all,
    benchmarks::rusqlite::all,
    benchmarks::sqlite::all,
}
