use criterion::{Criterion, criterion_group};
use std::time::Duration;
use tokio::runtime::Runtime;

use rsql_core::configuration::Configuration;
use rsql_core::shell::{Result, ShellArgs, ShellBuilder};

pub fn postgresql_benchmark(criterion: &mut Criterion) {
    criterion.bench_function("postgresql-embedded", |bencher| {
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");
        bencher
            .to_async(runtime)
            .iter(|| async { postgresql().await });
    });
}

async fn postgresql() -> Result<i32> {
    let args = ShellArgs {
        url: "postgresql://?embedded=true".to_string(),
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
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(10);
    targets = postgresql_benchmark
);
