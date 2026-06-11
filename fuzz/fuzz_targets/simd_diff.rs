// differential: SIMD-prefilter-accelerated find_all vs the scalar path.

#![no_main]

use libfuzzer_sys::fuzz_target;
use resharp::{force_scalar_scope, Regex};
use resharp_fuzz::hex;

fuzz_target!(|input: (&str, &[u8])| {
    let (pattern, haystack) = input;

    let accel = match Regex::new(pattern) {
        Ok(re) => re,
        Err(_) => return,
    };
    let accel_result = accel.find_all(haystack);

    let scalar_result = {
        let _scalar_scope = force_scalar_scope();
        match Regex::new(pattern) {
            Ok(re) => re.find_all(haystack),
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
