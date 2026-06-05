//! Regression coverage for the NO_MATCH sentinel reaching a `find_all` result.
//!
//! When the reverse pass proposes a match start that the forward scan then
//! rejects (forward end stays at `NO_MATCH`), the default `find_all` path used
//! to push `usize::MAX` as a `Match.end` (BUG-4) or, at the pos-0 branch, abort
//! the host (`assert_ne!`, BUG-2). It now skips the over-proposed candidate, and
//! a `debug_assert` keeps the underlying forward/reverse disagreement loud in
//! debug builds.
//!
//! On v0.6.9 the live trigger is the end-anchor complement in flags mode
//! (`~(_*$)`, `~(_*\z)`), which leaks `Match { end: usize::MAX }` before the fix;
//! a caller slicing `hay[start..end]` then reads out of bounds. (The pos-0 abort
//! site no longer reproduces from the earlier `.\W*b+`-style patterns on this
//! base, but the same guard is converted there too.)
//!
//! The behaviour differs by build profile, so each profile has its own test
//! (there is no debug-vs-release split in CI to rely on, so neither is left
//! unexercised):
//!
//! - release: `find_all` degrades gracefully (no out-of-bounds end, no abort).
//! - debug: the `debug_assert` fires on the same disagreement.

use resharp::{Regex, RegexOptions};

/// Flags mode matching the BUG-4 reproducer.
fn flags() -> RegexOptions {
    RegexOptions::default()
        .case_insensitive(true)
        .ignore_whitespace(true)
        .dot_matches_new_line(true)
        .multiline(false)
}

/// Release floor: the over-proposed candidate is skipped, so `find_all` returns
/// a bounded result instead of leaking the `usize::MAX` sentinel.
#[cfg(not(debug_assertions))]
#[test]
fn find_all_does_not_leak_no_match_sentinel() {
    for (pat, hay) in [
        (r"~(_*$)", "ab"),
        (r"~(_*\z)", "ab"),
        (r"~(_*\z)", "abc"),
    ] {
        let re = Regex::with_options(pat, flags()).unwrap();
        let matches = re.find_all(hay.as_bytes()).unwrap();
        for m in &matches {
            assert!(
                m.end <= hay.len() && m.start <= m.end,
                "pat={pat:?} hay={hay:?}: leaked sentinel / out-of-bounds match {m:?}"
            );
        }
    }
}

/// Debug invariant: the same forward/reverse disagreement that the release floor
/// skips silently must trip the `debug_assert` in debug builds, so the
/// underlying inconsistency stays visible rather than being papered over.
#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "forward scan found no end")]
fn find_all_debug_asserts_on_forward_reverse_disagreement() {
    let re = Regex::with_options(r"~(_*$)", flags()).unwrap();
    let _ = re.find_all(b"ab");
}
