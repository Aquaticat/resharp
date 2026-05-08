use resharp::Regex;
use std::path::Path;

#[test]
fn stream_toml() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("stream.toml");
    let content = std::fs::read_to_string(&path).unwrap();
    let table: toml::Value = content.parse().unwrap();
    let tests = table["test"].as_array().unwrap();
    for t in tests {
        let name = t.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let pattern = t["pattern"].as_str().unwrap();
        let input = t.get("input").and_then(|v| v.as_str()).unwrap_or("").as_bytes();
        let want: Vec<(usize, usize)> = t["matches"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| {
                let a = m.as_array().unwrap();
                (a[0].as_integer().unwrap() as usize, a[1].as_integer().unwrap() as usize)
            })
            .collect();
        let vs_find_all = t.get("vs_find_all").and_then(|v| v.as_bool()).unwrap_or(false);

        let re = Regex::new(pattern).unwrap_or_else(|e| panic!("{name}: compile: {e}"));
        let s = re.stream(input).unwrap();
        let got: Vec<(usize, usize)> = s.iter().map(|m| (m.start, m.end)).collect();
        assert_eq!(got, want, "name={name} pattern={pattern:?} input={input:?}");

        if vs_find_all {
            let f = re.find_all(input).unwrap();
            assert_eq!(s, f, "name={name} stream != find_all");
        }
    }
}

#[test]
fn test_stream_prefix_skip_helps() {
    let mut data = Vec::with_capacity(2_000_000);
    for _ in 0..50_000 {
        data.extend_from_slice(b"............................................");
        data.extend_from_slice(b"Id=\"42\" .");
    }
    let re = Regex::new(r#"Id="\d+""#).unwrap();
    let m = re.stream(&data).unwrap();
    assert_eq!(m.len(), 50_000);
}

#[test]
fn test_stream_with_callback() {
    let r = Regex::new(r"\d+").unwrap();
    let input = b"a12 b34 c5 d6789";
    let want = r.stream(input).unwrap();
    let mut got = Vec::new();
    r.stream_with(input, |m| got.push(m)).unwrap();
    assert_eq!(got, want);

    let mut count = 0usize;
    r.stream_with(input, |_| count += 1).unwrap();
    assert_eq!(count, want.len());

    let mut fired = false;
    r.stream_with(b"", |_| fired = true).unwrap();
    assert!(!fired);
}

#[test]
fn test_cross_chunk_boundary() {
    let r = resharp::Regex::new("abcdef").unwrap();
    let mut got = Vec::new();
    let mut state = resharp::StreamState::new();
    for chunk in [b"abc".as_slice(), b"def"] {
        state = r.stream_chunk(chunk, state, |e| got.push(e)).unwrap();
    }
    let want = r.stream_ends(b"abcdef").unwrap();
    assert_eq!(got, want);
}

#[test]
fn test_stream_chunk() {
    let r = Regex::new(r"\d+").unwrap();
    let input = b"a12 b34 c5 d6789";

    let want = r.stream_ends(input).unwrap();

    for chunk_size in [1, 2, 3, 4, 7, 16, input.len()] {
        let mut got = Vec::new();
        let mut state = resharp::StreamState::new();
        for chunk in input.chunks(chunk_size) {
            state = r.stream_chunk(chunk, state, |e| got.push(e)).unwrap();
        }
        assert_eq!(got, want, "chunk_size={chunk_size}");
    }
}


#[test]
fn seek_fwd_rev_cursor() {
    let re = Regex::new("a[bc]+d").unwrap();
    let input = b"xx abcd yy abbcd zz acd ww abd";
    let stream_matches: Vec<(usize, usize)> = re.stream(input).unwrap().iter().map(|m| (m.start, m.end)).collect();

    let mut fwd: Vec<usize> = Vec::new();
    let (mut s, mut p) = (Regex::SEEK_INITIAL, 0usize);
    while let Some((ns, end)) = re.seek_fwd(input, s, p).unwrap() {
        fwd.push(end);
        s = ns;
        p = end;
    }
    let want_ends: Vec<usize> = stream_matches.iter().map(|m| m.1).collect();
    assert_eq!(fwd, want_ends, "seek_fwd ends");

    let mut rev: Vec<usize> = Vec::new();
    let (mut s, mut p) = (Regex::SEEK_INITIAL, input.len());
    while let Some((ns, start)) = re.seek_rev(input, s, p).unwrap() {
        rev.push(start);
        s = ns;
        p = start;
    }
    let mut want_starts: Vec<usize> = stream_matches.iter().map(|m| m.0).collect();
    want_starts.reverse();
    assert_eq!(rev, want_starts, "seek_rev starts");
}

#[test]
fn seek_fwd_from_middle() {
    let re = Regex::new("lookaround").unwrap();
    let input = b"foo lookaround bar baz lookaround qux end";
    let mid = 20;
    let (_, end) = re.seek_fwd(input, Regex::SEEK_INITIAL, mid).unwrap().unwrap();
    assert_eq!(end, 33);
    assert_eq!(&input[end - 10..end], b"lookaround");
}

#[test]
fn seek_rev_from_middle() {
    let re = Regex::new("lookaround").unwrap();
    let input = b"foo lookaround bar baz lookaround qux end";
    let mid = 20;
    let (_, start) = re.seek_rev(input, Regex::SEEK_INITIAL, mid).unwrap().unwrap();
    assert_eq!(start, 4);
    assert_eq!(&input[start..start + 10], b"lookaround");
}

#[test]
fn seek_no_match() {
    let re = Regex::new("zzz").unwrap();
    let input = b"the quick brown fox jumps over the lazy dog";
    assert!(re.seek_fwd(input, Regex::SEEK_INITIAL, 10).unwrap().is_none());
    assert!(re.seek_rev(input, Regex::SEEK_INITIAL, 30).unwrap().is_none());
}

#[test]
fn seek_fwd_skips_match_before_pos() {
    let re = Regex::new("abcdef").unwrap();
    let input = b"xx abcdef yy abcdef zz";
    let (_, end) = re.seek_fwd(input, Regex::SEEK_INITIAL, 0).unwrap().unwrap();
    assert_eq!(end, 9);
    let (_, end) = re.seek_fwd(input, Regex::SEEK_INITIAL, 5).unwrap().unwrap();
    assert_eq!(end, 19);
    assert!(re.seek_fwd(input, Regex::SEEK_INITIAL, 20).unwrap().is_none());
}

#[test]
fn seek_fwd_with_class_pattern() {
    let re = Regex::new(r"\d+").unwrap();
    let input = b"abc 123 def 4567 ghi 89 jkl";
    let mut ends = Vec::new();
    let (mut s, mut p) = (Regex::SEEK_INITIAL, 8usize);
    while let Some((ns, e)) = re.seek_fwd(input, s, p).unwrap() {
        ends.push(e);
        s = ns;
        p = e;
    }
    assert_eq!(ends, vec![13, 14, 15, 16, 22, 23]);
}
