use std::{error::Error, fs};

use criterion::{criterion_group, criterion_main, Criterion};
use pop65::{assemble, dump_symtab, from_file};

fn test_s() -> Result<(), Box<dyn Error>> {
    let (bytes, symbols) = assemble(from_file("test.s")?)?;
    fs::write("test.prg", bytes)?;
    fs::write("test.sym", dump_symtab(symbols))?;
    Ok(())
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("test_s", |b| b.iter(|| test_s().unwrap()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
