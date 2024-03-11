use anyhow::Result;
use criterion::{criterion_group, criterion_main, Criterion};
use rsql_core::configuration::Configuration;
use rsql_core::version::full_version;

fn benchmarks(criterion: &mut Criterion) {
    bench_version(criterion).ok();
}

fn bench_version(criterion: &mut Criterion) -> Result<()> {
    criterion.bench_function("version", |bencher| {
        bencher.iter(|| {
            version().ok();
        });
    });

    Ok(())
}

fn version() -> Result<()> {
    let configuration = Configuration::default();
    let _ = full_version(&configuration)?;
    Ok(())
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = benchmarks
);
criterion_main!(benches);
