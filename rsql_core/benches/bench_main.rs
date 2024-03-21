use criterion::criterion_main;

mod benchmarks;

criterion_main! {
    benchmarks::postgresql::all,
    benchmarks::sqlite::all,
}
