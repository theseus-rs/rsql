use criterion::{criterion_group, Criterion};
use tokio::runtime::Runtime;

use rsql_core::configuration::Configuration;
use rsql_core::shell::{Result, ShellArgs, ShellBuilder};

pub fn rusqlite_benchmark(criterion: &mut Criterion) {
    criterion.bench_function("rusqlite", |bencher| {
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");
        bencher
            .to_async(runtime)
            .iter(|| async { rusqlite().await });
    });
}

async fn rusqlite() -> Result<i32> {
    let args = ShellArgs {
        url: "rusqlite://".to_string(),
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
    targets = rusqlite_benchmark
);
