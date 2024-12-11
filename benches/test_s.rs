use std::error::Error;

use criterion::{criterion_group, criterion_main, Criterion};
use pop65::{assemble, from_file};

fn test_s() -> Result<(), Box<dyn Error>> {
    let _info = assemble(from_file("test.s")?, false)?;
    Ok(())
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("test_s", |b| b.iter(|| test_s().unwrap()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
