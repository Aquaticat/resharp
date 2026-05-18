mod common;
use common::schemas::{DerivCase, DerivFile};
use resharp::{NodeId, RegexBuilder};
use resharp_algebra::nulls::Nullability;
use std::path::Path;

fn load_tests() -> Vec<DerivCase> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("deriv.toml");
    let content = std::fs::read_to_string(&path).unwrap();
    let file: DerivFile = toml::from_str(&content).unwrap();
    file.test
}

fn pos_mask(pos: usize, n: usize) -> Nullability {
    if n == 0 {
        Nullability::BEGIN.or(Nullability::END)
    } else if pos == 0 {
        Nullability::BEGIN
    } else if pos == n {
        Nullability::END
    } else {
        Nullability::CENTER
    }
}

fn walk_bytes(
    b: &mut RegexBuilder,
    mut node: NodeId,
    bytes: &[u8],
    expected: &[String],
    expected_nulls: Option<&[usize]>,
    dir: &str,
    name: &str,
) {
    if !expected.is_empty() {
        assert_eq!(
            bytes.len(),
            expected.len(),
            "input length must match {dir} expected length for {name}"
        );
    }
    let n = bytes.len();
    let report_null = |b: &mut RegexBuilder, node: NodeId, pos: usize, label: &str| -> bool {
        let mask = pos_mask(pos, n);
        let null = b.nullability(node).has(mask);
        eprintln!(
            "  [{}] {} pos={} mask={:?} nullable={}",
            dir, label, pos, mask, null
        );
        null
    };
    let mut got_nulls: Vec<usize> = Vec::new();
    if report_null(b, node, 0, "initial") {
        got_nulls.push(0);
    }
    for (i, byte) in bytes.iter().enumerate() {
        let der_mask = pos_mask(i, n);
        let tset = b.solver().u8_to_set_id(*byte);
        let tregex = b.der(node, der_mask).unwrap();
        let next = b.transition_term(tregex, tset);
        let pp = b.pp(next);
        eprintln!(
            "  [{}] step={} byte='{}' (0x{:02x}) der_mask={:?} node={:?} => {}",
            dir, i, *byte as char, byte, der_mask, next, pp
        );
        if let Some(exp) = expected.get(i) {
            if exp != "?" {
                assert_eq!(
                    pp, *exp,
                    "deriv pp mismatch: name={} dir={} step={} byte='{}'",
                    name, dir, i, *byte as char
                );
            }
        }
        node = next;
        if report_null(b, node, i + 1, "after") {
            got_nulls.push(i + 1);
        }
    }
    if let Some(exp) = expected_nulls {
        assert_eq!(
            got_nulls, exp,
            "nullability mismatch: name={} dir={}\n  got:      {:?}\n  expected: {:?}",
            name, dir, got_nulls, exp
        );
    }
}

#[test]
fn test_deriv_toml() {
    for tc in load_tests() {
        if tc.ignore {
            continue;
        }
        let mut b = RegexBuilder::new();
        let node = resharp_parser::parse_ast(&mut b, &tc.pattern).unwrap();

        if !tc.rev.is_empty() || tc.rev_nulls.is_some() {
            let rev = b.reverse(node).unwrap();
            let rev = b.normalize_rev(rev).unwrap();
            let rev = b.mk_concat(NodeId::TS, rev);

            eprintln!(
                "\n[{}] rev initial: node={:?} pp={}",
                tc.name,
                rev,
                b.pp(rev)
            );
            let bytes: Vec<u8> = tc.input.as_bytes().iter().rev().copied().collect();
            walk_bytes(
                &mut b,
                rev,
                &bytes,
                &tc.rev,
                tc.rev_nulls.as_deref(),
                "rev",
                &tc.name,
            );
        }

        if !tc.fwd.is_empty() || tc.fwd_nulls.is_some() {
            eprintln!(
                "\n[{}] fwd initial: node={:?} kind={:?} pp={}",
                tc.name,
                node,
                b.get_kind(node),
                b.pp(node)
            );
            let bytes: Vec<u8> = tc.input.as_bytes().to_vec();
            walk_bytes(
                &mut b,
                node,
                &bytes,
                &tc.fwd,
                tc.fwd_nulls.as_deref(),
                "fwd",
                &tc.name,
            );
        }
    }
}
