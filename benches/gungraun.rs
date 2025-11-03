use gungraun::{Callgrind, CallgrindMetrics, EntryPoint, EventKind, LibraryBenchmarkConfig};
use gungraun::{library_benchmark, library_benchmark_group, main};

use crate::runner::{ParsedScript, Runner};

mod runner;

#[cfg(feature = "bench-parse")]
fn setup_parse(source_str: &str) -> (Runner, &str) {
    (Runner::new(true), source_str)
}

fn setup_exec(source_str: &str) -> ParsedScript {
    Runner::new(true).parse_script(source_str, true)
}

#[library_benchmark]
#[bench::setup(true)]
fn bench_vmsetup(gc: bool) -> Runner {
    Runner::new(gc)
}

macro_rules! bench_harness {
    ($($ID:ident : $name:literal,)*) => {
        $(
            mod $ID {
                pub(super) static CODE: &str = include_str!(concat!("scripts/", $name));
            }
        )*

        #[cfg(feature = "bench-parse")]
        #[library_benchmark(
            config = parse_config(),
            setup = setup_parse
        )]
        $(#[bench::$ID($ID::CODE)])*
        fn bench_parse(input: (Runner, &str)) {
            let (runner, script) = input;
            runner.parse_script(script, true);
        }

        #[library_benchmark(
            config = exec_config(),
            setup = setup_exec
        )]
        $(#[bench::$ID($ID::CODE)])*
        fn bench_exec(script: ParsedScript) {
            script.run();
        }
    };
}

bench_harness!(
    boa_arith : "boa/arithmetic_operations.js",
    boa_array_access : "boa/array_access.js",
    boa_array_create : "boa/array_create.js",
    boa_array_pop : "boa/array_pop.js",
    boa_bool_obj_access : "boa/boolean_object_access.js",
    boa_clean : "boa/clean_js.js",
    boa_fib : "boa/fibonacci.js",
    boa_for : "boa/for_loop.js",
    boa_min : "boa/mini_js.js",
    boa_number_obj_access : "boa/number_object_access.js",
    boa_obj_create : "boa/object_creation.js",
    boa_obj_prop_access_const : "boa/object_prop_access_const.js",
    boa_obj_prop_access_dyn : "boa/object_prop_access_dyn.js",
    boa_regexp : "boa/regexp.js",
    boa_regexp_create : "boa/regexp_creation.js",
    boa_regexp_lit : "boa/regexp_literal.js",
    boa_regexp_lit_create : "boa/regexp_literal_creation.js",
    boa_str_cmp : "boa/string_compare.js",
    boa_str_concat : "boa/string_concat.js",
    boa_str_cp : "boa/string_copy.js",
    boa_str_obj_access : "boa/string_object_access.js",
    boa_symb_create : "boa/symbol_creation.js",
    simple_array : "simple/array.js",
    simple_binary_trees : "simple/binary-trees.js",
    simple_count : "simple/count.js",
    simple_fib_slow : "simple/fibonacci-slow.js",
    simple_fib_fast : "simple/fibonacci-fast.js",
);

library_benchmark_group!(
   name = bench_vmsetup_group;
   benchmarks = bench_vmsetup
);

#[cfg(feature = "bench-parse")]
library_benchmark_group!(
   name = bench_parse_group;
   benchmarks = bench_parse
);

library_benchmark_group!(
   name = bench_exec_group;
   benchmarks = bench_exec
);

fn callgrind_config() -> Callgrind {
    let mut cfg = Callgrind::with_args(["branch-sim=yes", "cacheuse=yes"]);
    cfg.format([
        EventKind::EstimatedCycles.into(),
        EventKind::Ir.into(),
        EventKind::TotalRW.into(),
        CallgrindMetrics::CacheMissRates,
        CallgrindMetrics::CacheMisses,
        CallgrindMetrics::CacheUse,
        EventKind::Bcm.into(),
        EventKind::Bim.into(),
    ]);
    cfg
}

fn parse_config() -> LibraryBenchmarkConfig {
    let mut cfg = LibraryBenchmarkConfig::default();
    cfg.tool(callgrind_config().entry_point(EntryPoint::Custom(
        "*::runner::parse_script_entry".to_owned(),
    )));
    cfg
}

fn exec_config() -> LibraryBenchmarkConfig {
    let mut cfg = LibraryBenchmarkConfig::default();
    cfg.tool(callgrind_config().entry_point(EntryPoint::Custom(
        "*::runner::script_evaluation_entry".to_owned(),
    )));
    cfg
}

#[cfg(feature = "bench-parse")]
main!(
    library_benchmark_groups = bench_vmsetup_group,
    bench_parse_group,
    bench_exec_group,
);
#[cfg(not(feature = "bench-parse"))]
main!(
    library_benchmark_groups = bench_vmsetup_group,
    bench_exec_group,
);
