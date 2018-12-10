use libtmplgen::*;
use std::env::set_var;
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_gen_perl(c: &mut Criterion) {
    let dir = tempfile::tempdir().unwrap();
    set_var("XBPS_DISTDIR", dir.path());

    c.bench_function(
        "bench_gen_perl",
        |b| b.iter(|| {
            TmplBuilder::new("Module::Build")
                .get_type()
                .unwrap()
                .get_info()
                .unwrap()
                .generate(true)
                .unwrap();
        }),
    );

    dir.close().unwrap();
}

fn bench_built_in(c: &mut Criterion) {
    c.bench_function(
        "bench_gen_perl",
        |b| b.iter(|| {
            TmplBuilder::new("perl")
            .set_type(PkgType::PerlDist)
            .is_built_in()
            .unwrap()
        }),
    );
}

fn bench_gen_deps_perl(c: &mut Criterion) {
    c.bench_function(
        "bench_gen_deps_perl",
        |b| b.iter(|| {
            TmplBuilder::new("Moose")
                .set_type(PkgType::PerlDist)
                .get_info()
                .unwrap()
                .gen_deps(None)
                .unwrap();
        }),
    );
}

criterion_group!{
    name = benches;
    config = Criterion::default().sample_size(10).nresamples(50).without_plots();
    targets = bench_built_in, bench_gen_deps_perl, bench_gen_perl
}
criterion_main!(benches);