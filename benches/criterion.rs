// Inspired by `boa`s criterion benchmark harness
// <https://github.com/boa-dev/boa/blob/main/core/engine/benches/full.rs>

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};

mod runner;

fn bench_vmstartup(c: &mut Criterion) {
    c.bench_function("VM Startup", |b| b.iter(|| runner::Runner::new(true)));
}

macro_rules! bench_harness {
    ($($name:literal,)*) => {
        fn bench_parse(c: &mut Criterion) {
            $(
                {
                    static CODE: &str = include_str!(concat!("scripts/", $name));

                    c.bench_function(concat!($name, " (Parse)"), move |b| {
                        b.iter_batched(
                            || { runner::Runner::new(true) },
                            |runner| { runner.parse_script(CODE, true) },
                            BatchSize::PerIteration,
                        )
                    });
                }
            )*
        }

        fn bench_exec(c: &mut Criterion) {
            $(
                {
                    static CODE: &str = include_str!(concat!("scripts/", $name));

                    c.bench_function(concat!($name, " (Exec)"), move |b| {
                        b.iter_batched(
                            || { runner::Runner::new(true).parse_script(CODE, true) },
                            |script| { script.run() },
                            BatchSize::PerIteration,
                        )
                    });
                }
            )*
        }
    };
}

bench_harness!(
    "boa/arithmetic_operations.js",
    "boa/array_access.js",
    "boa/array_create.js",
    "boa/array_pop.js",
    "boa/boolean_object_access.js",
    "boa/clean_js.js",
    "boa/fibonacci.js",
    "boa/for_loop.js",
    "boa/mini_js.js",
    "boa/number_object_access.js",
    "boa/object_creation.js",
    "boa/object_prop_access_const.js",
    "boa/object_prop_access_dyn.js",
    "boa/regexp.js",
    "boa/regexp_creation.js",
    "boa/regexp_literal.js",
    "boa/regexp_literal_creation.js",
    "boa/string_compare.js",
    "boa/string_concat.js",
    "boa/string_copy.js",
    "boa/string_object_access.js",
    "boa/symbol_creation.js",
    "simple/array.js",
    "simple/binary-trees.js",
    "simple/count.js",
    "simple/fibonacci-slow.js",
    "simple/fibonacci-fast.js",
);

criterion_group!(bench_vmstartup_group, bench_vmstartup);
criterion_group!(bench_parse_group, bench_parse);
criterion_group!(bench_exec_group, bench_exec);

criterion_main!(bench_vmstartup_group, bench_parse_group, bench_exec_group);
