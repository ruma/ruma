// `cargo bench` works, but if you use `cargo bench -- --save-baseline <name>`
// or pass any other args to it, it fails with the error
// `cargo bench unknown option --save-baseline`.
// To pass args to criterion, use this form
// `cargo bench --bench <name of the bench> -- --save-baseline <name>`.

use criterion::{criterion_group, criterion_main, Criterion};

fn state_res(c: &mut Criterion) {
    c.bench_function("resolve state of 10 events", |b| b.iter(|| {}));
}

criterion_group!(benches, state_res,);

criterion_main!(benches);
