//! Release-only regression coverage for the NO_MATCH sentinel reaching a
//! `find_all` result.
//!
//! When the reverse pass proposes a match start that the forward scan then
//! rejects (forward end stays at `NO_MATCH`), the default `find_all` path used
//! to abort the host (`assert_ne!`, BUG-2) or push `usize::MAX` as a `Match.end`
//! (BUG-4). It now skips the over-proposed candidate. In debug builds a
//! `debug_assert` still fires on that disagreement to keep the underlying
//! forward/reverse inconsistency visible, so this release floor is checked only
//! when debug assertions are off.
#![cfg(not(debug_assertions))]

use resharp::{Regex, RegexOptions};

#[test]
fn find_all_does_not_abort_or_leak_no_match_sentinel() {
    // Default mode: these aborted at engine.rs:960 (BUG-2) or leaked
    // end == usize::MAX (BUG-4). The forward scan rejects every candidate, so
    // the correct result is empty (matches the dotnet reference).
    for (pat, hay) in [
        (r".\W*b+", "ba"),
        (r"\S+b", "b'_"),
        (r"\Bb+", "ba"),
        (r"(?<=[^a])b+", "ba"),
    ] {
        let re = Regex::new(pat).unwrap();
        let matches = re.find_all(hay.as_bytes()).unwrap();
        for m in &matches {
            assert!(
                m.end <= hay.len() && m.start <= m.end,
                "pat={pat:?} hay={hay:?}: unbounded match {m:?}"
            );
        }
        assert!(
            matches.is_empty(),
            "pat={pat:?} hay={hay:?}: expected no match, got {matches:?}"
        );
    }

    // Flags mode end-anchor complement: leaked usize::MAX (BUG-4). The surviving
    // candidate must be in bounds.
    let flags = RegexOptions::default()
        .case_insensitive(true)
        .ignore_whitespace(true)
        .dot_matches_new_line(true)
        .multiline(false);
    let re = Regex::with_options(r"~(_*$)", flags).unwrap();
    for m in re.find_all(b"ab").unwrap().iter() {
        assert!(m.end <= 2 && m.start <= m.end, "leaked sentinel: {m:?}");
    }
}
