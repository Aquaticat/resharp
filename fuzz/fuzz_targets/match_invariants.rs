// self-consistency of matching: for any pattern that compiles and any haystack,
// `find_all` / `find_anchored` / `is_match` / `stream` must satisfy the
// invariants the API documents, with no oracle required.
//
//   * every match is a valid `[start, end)` slice of the haystack
//     (`start <= end <= len`);
//   * `find_all` returns leftmost-first, non-overlapping matches
//     (`match[i].start >= match[i-1].end`);
//   * `find_all` is non-empty  <=>  `is_match` is true;
//   * `find_anchored`, when it matches, matches at offset 0, and implies
//     `is_match`;
//   * `find_all` empty implies `stream` empty, and for purely zero-width
//     result sets `stream` agrees with `find_all` exactly (leftmost-shortest
//     equals leftmost-longest when every match is zero-width);
//   * the default and hardened engines return identical `find_all` results:
//     hardening swaps the forward-scan algorithm, not the language.

#![no_main]

use libfuzzer_sys::fuzz_target;
use resharp::{Match, Regex, RegexOptions};
use resharp_fuzz::hex;

fuzz_target!(|input: (&str, &[u8])| {
    let (pattern, haystack) = input;

    let mut per_engine: [Option<Vec<Match>>; 2] = [None, None];

    for (engine_idx, opts) in [
        RegexOptions::default(),
        RegexOptions::default().hardened(true),
    ]
    .into_iter()
    .enumerate()
    {
        let re = match Regex::with_options(pattern, opts) {
            Ok(re) => re,
            Err(_) => continue,
        };

        let matches = match re.find_all(haystack) {
            Ok(matches) => matches,
            Err(_) => continue,
        };

        let mut prev_end = 0usize;
        for (i, m) in matches.iter().enumerate() {
            assert!(
                m.start <= m.end && m.end <= haystack.len(),
                "out-of-bounds match {m:?} (haystack len {}) \
                 pattern={pattern:?} haystack_hex={}",
                haystack.len(),
                hex(haystack),
            );
            if i > 0 {
                assert!(
                    m.start >= prev_end,
                    "overlapping / unsorted match {m:?} after end {prev_end} \
                     pattern={pattern:?} haystack_hex={}",
                    hex(haystack),
                );
            }
            prev_end = m.end;
        }

        if let Ok(is_match) = re.is_match(haystack) {
            assert_eq!(
                !matches.is_empty(),
                is_match,
                "find_all/is_match disagree (find_all len {}, is_match {is_match}) \
                 pattern={pattern:?} haystack_hex={}",
                matches.len(),
                hex(haystack),
            );
        }

        if let Ok(Some(m)) = re.find_anchored(haystack) {
            assert!(
                m.start == 0 && m.end <= haystack.len(),
                "find_anchored returned non-anchored / out-of-bounds match {m:?} \
                 (haystack len {}) pattern={pattern:?} haystack_hex={}",
                haystack.len(),
                hex(haystack),
            );
            if let Ok(is_match) = re.is_match(haystack) {
                assert!(
                    is_match,
                    "find_anchored={m:?} but is_match=false \
                     pattern={pattern:?} haystack_hex={}",
                    hex(haystack),
                );
            }
        }

        if let Ok(stream) = re.stream(haystack) {
            if matches.is_empty() {
                assert!(
                    stream.is_empty(),
                    "find_all empty but stream={stream:?} \
                     pattern={pattern:?} haystack_hex={}",
                    hex(haystack),
                );
            } else if matches.iter().all(|m| m.start == m.end) {
                // leftmost-shortest equals leftmost-longest when every match
                // is zero-width, so the two enumerations must coincide.
                assert_eq!(
                    stream,
                    matches,
                    "stream/find_all disagree on a zero-width result set \
                     pattern={pattern:?} haystack_hex={}",
                    hex(haystack),
                );
            }
        }

        per_engine[engine_idx] = Some(matches);
    }

    if let [Some(default_ms), Some(hardened_ms)] = &per_engine {
        assert_eq!(
            default_ms,
            hardened_ms,
            "default/hardened find_all diverge \
             pattern={pattern:?} haystack_hex={}",
            hex(haystack),
        );
    }
});
