// differential `is_match` against the `regex` crate, over the syntax subset
// where the two engines share semantics (see `DiffPattern`). a disagreement is
// a correctness finding in one of the engines (almost always resharp, since the
// regex crate is the more battle-tested oracle).
//
// only existence is compared, never match offsets: resharp is leftmost-longest
// and regex is leftmost-greedy, so positions legitimately differ while
// `is_match` must not. resharp runs in `UnicodeMode::Ascii` and regex runs with
// `.unicode(false)` so `.` and negated classes are byte-oriented on both sides.

#![no_main]

use libfuzzer_sys::fuzz_target;
use resharp::{Regex, RegexOptions, UnicodeMode};
use resharp_fuzz::{hex, DiffPattern};

#[derive(Debug, arbitrary::Arbitrary)]
struct DiffInput {
    pattern: DiffPattern,
    haystack: Vec<u8>,
}

fuzz_target!(|input: DiffInput| {
    let pattern = &input.pattern.0;
    let haystack = &input.haystack;

    let rs = match Regex::with_options(
        pattern,
        RegexOptions::default().unicode(UnicodeMode::Ascii),
    ) {
        Ok(rs) => rs,
        Err(_) => return,
    };
    let re = match regex::bytes::RegexBuilder::new(pattern).unicode(false).build() {
        Ok(re) => re,
        Err(_) => return,
    };

    let rs_match = match rs.is_match(haystack) {
        Ok(b) => b,
        Err(_) => return,
    };
    let re_match = re.is_match(haystack);

    assert_eq!(
        rs_match, re_match,
        "is_match divergence: resharp={rs_match} regex={re_match} \
         pattern={pattern:?} haystack_hex={}",
        hex(haystack),
    );
});
