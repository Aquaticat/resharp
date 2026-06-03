// robustness of `Regex::with_options`: an arbitrary pattern string compiled
// under one configuration from the option sweep must never panic, abort, or
// hang. parse / capacity / size rejections are expected and returned as `Err`;
// only a crash (panic, stack overflow, OOM, ASAN report) is a finding.
//
// this is the primary target: the known resharp defect class (intersection
// over alternation, intersection + quantifier, nullability assertions) all
// surface here, at compile time, inside `Regex::new` / `with_options`.
//
// one compile per unit: the first input byte selects which `option_sweep()`
// config to use, and the rest is the pattern (decoded as the longest valid
// UTF-8 prefix, matching how a `&str` fuzz argument is produced). compiling
// under a single option per unit keeps the libFuzzer `-timeout` watchdog
// measuring one `Regex::with_options` call. compiling all six per unit instead
// multiplied a benign sub-second compile by six under ASAN, tripping
// `-timeout=10` on patterns that are not actually slow (see the resharp
// troubleshooting doc, "Compile-time timeouts on small patterns").

#![no_main]

use libfuzzer_sys::fuzz_target;
use resharp::Regex;
use resharp_fuzz::option_sweep;

// longest valid-UTF-8 prefix of `bytes`.
fn decode(bytes: &[u8]) -> &str {
    match std::str::from_utf8(bytes) {
        Ok(s) => s,
        Err(e) => std::str::from_utf8(&bytes[..e.valid_up_to()]).unwrap(),
    }
}

fuzz_target!(|data: &[u8]| {
    let Some((&selector, rest)) = data.split_first() else {
        return;
    };
    let sweep = option_sweep();
    let idx = selector as usize % sweep.len();
    let opts = sweep.into_iter().nth(idx).unwrap();
    // discard the result: `Ok` means it compiled, `Err` is an expected
    // rejection. a crash is what libFuzzer records.
    let _ = Regex::with_options(decode(rest), opts);
});
