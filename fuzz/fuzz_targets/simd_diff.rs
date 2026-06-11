// differential: the SIMD-prefilter-accelerated find_all drivers vs the scalar
// paths. prefix acceleration is decided while the Regex is built
// (build_fwd_prefix returns None when has_simd() is false), so each side
// compiles its own Regex with the override set accordingly; the two must then
// agree exactly on every (pattern, haystack).
//
// this is the oracle that catches prefilter-driver soundness bugs the SIMD
// intrinsic unit tests cannot see, e.g. `^$` over "\n\n" returning
// [0:0, 2:2] from the accelerated driver while the scalar path returns the
// correct [0:0, 1:1, 2:2].

#![no_main]

use libfuzzer_sys::fuzz_target;
use resharp::{force_scalar, Regex};
use resharp_fuzz::hex;

fuzz_target!(|input: (&str, &[u8])| {
    let (pattern, haystack) = input;

    force_scalar(false);
    let accel = match Regex::new(pattern) {
        Ok(re) => re,
        Err(_) => return,
    };
    let accel_result = accel.find_all(haystack);

    force_scalar(true);
    let scalar_re = Regex::new(pattern);
    force_scalar(false);
    let scalar = match scalar_re {
        Ok(re) => re,
        // both sides parsed the same pattern; compile must not depend on the
        // override.
        Err(e) => panic!("scalar compile failed after accel compile succeeded: {e:?} pattern={pattern:?}"),
    };
    let scalar_result = scalar.find_all(haystack);

    match (accel_result, scalar_result) {
        (Ok(a), Ok(s)) => assert_eq!(
            a,
            s,
            "simd/scalar find_all diverge: accel={a:?} scalar={s:?} \
             pattern={pattern:?} haystack_hex={}",
            hex(haystack),
        ),
        (Err(_), Err(_)) => {}
        (a, s) => panic!(
            "simd/scalar error-status diverges: accel_ok={} scalar_ok={} \
             pattern={pattern:?} haystack_hex={}",
            a.is_ok(),
            s.is_ok(),
            hex(haystack),
        ),
    }
});
