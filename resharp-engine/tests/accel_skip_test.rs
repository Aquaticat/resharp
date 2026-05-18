mod common;
use common::schemas::EngineFile;
use resharp::{Regex, RegexOptions};
use std::path::Path;

#[test]
#[ignore = "slow in debug; run with --ignored or in release"]
fn accel_skip_lazy() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("accel_skip.toml");
    let content = std::fs::read_to_string(&path).unwrap();
    let file: EngineFile = toml::from_str(&content).unwrap();
    for tc in file.test {
        let re = Regex::with_options(
            &tc.pattern,
            RegexOptions {
                max_dfa_capacity: 10000,
                ..Default::default()
            },
        )
        .unwrap();
        let matches = re.find_all(tc.input.as_bytes()).unwrap();
        let result: Vec<[usize; 2]> = matches.iter().map(|m| [m.start, m.end]).collect();
        assert_eq!(
            result, tc.matches,
            "lazy: pattern={:?}, input={:?}",
            tc.pattern, tc.input
        );
    }
}
