use resharp::{Regex, RegexOptions, UnicodeMode};

fn measure(label: &str, pat: &str, opts: RegexOptions, hay: &[u8]) {
    let t0 = std::time::Instant::now();
    let re = match Regex::with_options(pat, opts) {
        Ok(r) => r,
        Err(e) => {
            println!("{:<26} err: {e}", label);
            return;
        }
    };
    let compile_s = t0.elapsed().as_secs_f64();

    let t1 = std::time::Instant::now();
    let _ = re.is_match(hay).unwrap();
    let m1_s = t1.elapsed().as_secs_f64();

    let t2 = std::time::Instant::now();
    let _ = re.is_match(hay).unwrap();
    let m2_s = t2.elapsed().as_secs_f64();

    println!("{:<26} compile={:.4}s  match1={:.4}s  match2={:.4}s", label, compile_s, m1_s, m2_s);
}

fn full() -> RegexOptions {
    RegexOptions::default().unicode(UnicodeMode::Full)
}

fn dflt() -> RegexOptions {
    RegexOptions::default()
}

fn main() {
    let hay16k: Vec<u8> = (0..16384u32).map(|i| (i % 256) as u8).collect();

    println!("--- diverse 16K (first vs second match) ---");
    let cases: &[(&str, &str, fn() -> RegexOptions)] = &[
        (r"$?\w  full", r"$?\w", full),
        (r"$\w   full", r"$\w", full),
        (r"\w    full", r"\w", full),
        (r"$?\W  full", r"$?\W", full),
        (r"$?\d  full", r"$?\d", full),
        (r"$?\w  dflt", r"$?\w", dflt),
    ];
    for (label, pat, opts) in cases {
        measure(label, pat, opts(), &hay16k);
    }

    println!("\n--- scaling with N diverse bytes ---");
    for n in [256, 512, 1024, 2048, 4096] {
        let hay: Vec<u8> = (0..n as u32).map(|i| (i % 256) as u8).collect();
        measure(&format!("$?\\w full N={n}"), r"$?\w", full(), &hay);
    }
}
