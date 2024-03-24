use criterion::{criterion_group, Criterion};
use tokio::runtime::Runtime;

use rsql_core::configuration::Configuration;
use rsql_core::shell::{Result, ShellArgs, ShellBuilder};

pub fn sqlite_benchmark(criterion: &mut Criterion) {
    criterion.bench_function("sqlite", |bencher| {
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");
        bencher.to_async(runtime).iter(|| async { sqlite().await });
    });
}

async fn sqlite() -> Result<()> {
    let args = ShellArgs {
        url: "sqlite::memory:".to_string(),
        commands: vec!["SELECT 1".to_string()],
        ..ShellArgs::default()
    };
    let configuration = Configuration::default();
    let mut output = Vec::new();
    let mut shell = ShellBuilder::new(&mut output)
        .with_configuration(configuration)
        .build();
    shell.execute(&args).await
}

criterion_group!(
    name = all;
    config = Criterion::default().sample_size(10);
    targets = sqlite_benchmark
);
