// differential: the SIMD-prefilter-accelerated find_all drivers vs the scalar
// paths. the override must cover the whole scalar side, not just
// construction: prefix acceleration is decided while the Regex is built
// (build_fwd_prefix returns None when has_simd() is false), and the lazy DFA
// also consults has_simd() per newly built state DURING the scan
// (try_build_skip_simd in ldfa.rs), so the guard holds force_scalar(true)
// through find_all and resets on drop.
//
// this is the oracle that catches prefilter-driver soundness bugs the SIMD
// intrinsic unit tests cannot see, e.g. `^$` over "\n\n" returning
// [0:0, 2:2] from the accelerated driver while the scalar path returns the
// correct [0:0, 1:1, 2:2].

#![no_main]

use libfuzzer_sys::fuzz_target;
use resharp::{force_scalar, Regex};
use resharp_fuzz::hex;

// scoped force_scalar: enabled on construction, reset on drop, so every exit
// path (including assertion unwinds) restores the accelerated default.
struct ForceScalar;

impl ForceScalar {
    fn new() -> Self {
        force_scalar(true);
        ForceScalar
    }
}

impl Drop for ForceScalar {
    fn drop(&mut self) {
        force_scalar(false);
    }
}

fuzz_target!(|input: (&str, &[u8])| {
    let (pattern, haystack) = input;

    let accel = match Regex::new(pattern) {
        Ok(re) => re,
        Err(_) => return,
    };
    let accel_result = accel.find_all(haystack);

    let scalar_result = {
        let _scalar_scope = ForceScalar::new();
        match Regex::new(pattern) {
            // both construction and the scan run under the override; lazy DFA
            // states built mid-scan must also take the scalar path.
            Ok(re) => re.find_all(haystack),
            // both sides parsed the same pattern; compile must not depend on
            // the override.
            Err(e) => panic!(
                "scalar compile failed after accel compile succeeded: {e:?} pattern={pattern:?}"
            ),
        }
    };

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
