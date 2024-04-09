use criterion::{criterion_group, Criterion};
use tokio::runtime::Runtime;

use rsql_core::configuration::Configuration;
use rsql_core::shell::{Result, ShellArgs, ShellBuilder};

pub fn duckdb_benchmark(criterion: &mut Criterion) {
    criterion.bench_function("duckdb", |bencher| {
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");
        bencher.to_async(runtime).iter(|| async { duckdb().await });
    });
}

async fn duckdb() -> Result<i32> {
    let args = ShellArgs {
        url: "duckdb://?memory=true".to_string(),
        commands: vec!["SELECT 1".to_string()],
        ..ShellArgs::default()
    };
    let configuration = Configuration::default();
    let mut shell = ShellBuilder::default()
        .with_configuration(configuration)
        .build();
    shell.execute(&args).await
}

criterion_group!(
    name = all;
    config = Criterion::default().sample_size(10);
    targets = duckdb_benchmark
);
