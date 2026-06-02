mod common;
use common::schemas::RevNullsFile;
use resharp::Regex;
use std::path::Path;

#[test]
fn test_rev_nulls_toml() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("rev_nulls.toml");
    let content = std::fs::read_to_string(&path).unwrap();
    let file: RevNullsFile = toml::from_str(&content).unwrap();
    for tc in file.test {
        if tc.ignore {
            continue;
        }
        let re = Regex::new(&tc.pattern).unwrap_or_else(|e| {
            panic!(
                "name={} pattern={:?}: compile error: {}",
                tc.name, tc.pattern, e
            )
        });
        let got = re.collect_rev_nulls_debug(tc.input.as_bytes());
        for i in 1..got.len() {
            assert!(
                got[i] <= got[i - 1],
                "rev nulls not sorted descending at [{}]: {} > {} (name={}, pattern={:?}, got={:?})",
                i, got[i], got[i - 1], tc.name, tc.pattern, got
            );
        }
        assert_eq!(
            got, tc.rev_nulls,
            "name={} pattern={:?} input={:?}",
            tc.name, tc.pattern, tc.input
        );
    }
}
