use anyhow::{bail, Result};
use criterion::{criterion_group, criterion_main, Criterion};
use std::process::Command;
use std::time::Duration;

fn benchmarks(criterion: &mut Criterion) {
    bench_version(criterion).ok();
}

fn bench_version(criterion: &mut Criterion) -> Result<()> {
    criterion.bench_function("cli-version", |bencher| {
        bencher.iter(|| {
            version().ok();
        });
    });

    Ok(())
}

fn version() -> Result<()> {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--release")
        .arg("--")
        .arg("--version")
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        bail!("Failed to execute command");
    }

    Ok(())
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(30))
        .sample_size(10);
    targets = benchmarks
);
criterion_main!(benches);
