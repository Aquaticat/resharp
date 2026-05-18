mod common;
use common::schemas::AutoHardenFile;
use resharp::{Regex, RegexOptions};
use std::path::Path;

#[test]
fn auto_harden_toml() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("auto_harden.toml");
    let content = std::fs::read_to_string(&path).unwrap();
    let file: AutoHardenFile = toml::from_str(&content).unwrap();
    for tc in file.test {
        let re = Regex::new(&tc.pattern).expect("pattern compiles");
        assert_eq!(
            re.is_hardened(),
            tc.hardened,
            "pattern={:?}: expected is_hardened={}, got {}",
            tc.pattern,
            tc.hardened,
            re.is_hardened()
        );
        if let Some(fwd) = tc.fwd {
            assert_eq!(
                re.has_fwd_prefix(),
                fwd,
                "pattern={:?}: expected has_fwd_prefix={}, got {}",
                tc.pattern,
                fwd,
                re.has_fwd_prefix()
            );
        }

        if tc.hardened {
            let hardened =
                Regex::with_options(&tc.pattern, RegexOptions::default().hardened(true)).unwrap();
            let inputs: &[&[u8]] = &[b"", b"aaaaaaaa", b"abcdefg", b"|  |\n| a |\n|  |"];
            for input in inputs {
                assert_eq!(
                    re.find_all(input).unwrap(),
                    hardened.find_all(input).unwrap(),
                    "pattern={:?} input={:?}",
                    tc.pattern,
                    input
                );
            }
        }
    }
}
