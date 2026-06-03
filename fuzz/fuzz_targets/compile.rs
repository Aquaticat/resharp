// robustness of `Regex::with_options`: an arbitrary pattern string compiled
// under every configuration in the option sweep must never panic, abort, or
// hang. parse / capacity / size rejections are expected and returned as `Err`;
// only a crash (panic, stack overflow, OOM, ASAN report) is a finding.
//
// this is the primary target: the known resharp defect class (intersection
// over alternation, intersection + quantifier, nullability assertions) all
// surface here, at compile time, inside `Regex::new` / `with_options`.

#![no_main]

use libfuzzer_sys::fuzz_target;
use resharp::Regex;
use resharp_fuzz::option_sweep;

fuzz_target!(|pattern: &str| {
    for opts in option_sweep() {
        // discard the result: `Ok` means it compiled, `Err` is an expected
        // rejection. a crash is what libFuzzer records.
        let _ = Regex::with_options(pattern, opts);
    }
});
