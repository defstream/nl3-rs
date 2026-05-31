//! Benchmarks for `Nl3::parse()` across representative input classes.
//!
//! The client is built once, outside the timed loop, so we measure parsing —
//! not grammar compilation. Run with `cargo bench`.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use nl3::Nl3;

fn client() -> Nl3 {
    Nl3::builder()
        .grammar([
            "users follow users",
            "users mention content",
            "users create messages",
            "users send messages",
            "users receive messages",
            "users message users",
        ])
        .vocabulary([
            ("follow", "follow"),
            ("stalk", "follow"),
            ("watch", "follow"),
            ("creat", "create"),
            ("made", "create"),
            ("wrote", "create"),
            ("send", "send"),
            ("sent", "send"),
            ("mail", "send"),
            ("retriev", "receive"),
            ("receiv", "receive"),
            ("reciev", "receive"),
            ("got", "receive"),
            ("messag", "message"),
            ("msg", "message"),
            ("contact", "message"),
        ])
        .build()
}

fn bench_parse(c: &mut Criterion) {
    let nl3 = client();

    let mut group = c.benchmark_group("parse");

    // Fully-typed phrase: predicate found early, both types explicit.
    group.bench_function("typed", |b| {
        b.iter(|| nl3.parse(black_box("user bob contacts user jill")))
    });

    // Synonym phrase: exercises the stemmer (contacted -> contact -> message).
    group.bench_function("synonym", |b| {
        b.iter(|| nl3.parse(black_box("user bob messaged user jill")))
    });

    // Inferred types: both ends omitted, resolved from the grammar.
    group.bench_function("inferred", |b| {
        b.iter(|| nl3.parse(black_box("bob contacts jill")))
    });

    // Invalid phrase: walks every token without finding a predicate.
    group.bench_function("invalid", |b| {
        b.iter(|| nl3.parse(black_box("dog jim hates cat sue")))
    });

    group.finish();
}

criterion_group!(benches, bench_parse);
criterion_main!(benches);
