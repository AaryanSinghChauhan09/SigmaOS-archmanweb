//! Integration tests for sigmaos-archmanweb.

use archmanweb::{Package, build_index, parse_man_filename, search};

#[test]
fn test_index_and_search() {
    let pkgs = vec![
        Package { repo: "core".into(), name: "coreutils".into(), version: "9.0".into(), arch: "x86_64".into(), description: "GNU core utils".into() },
        Package { repo: "extra".into(), name: "lsof".into(), version: "4.9".into(), arch: "x86_64".into(), description: "list open files".into() },
    ];
    let files = vec![
        ("coreutils".to_string(), "usr/share/man/man1/ls.1.gz".to_string(), ".TH LS 1\n.SH NAME\nls \\- list directory contents\n".to_string()),
        ("lsof".to_string(), "usr/share/man/man8/lsof.8".to_string(), ".TH LSOF 8\n.SH NAME\nlsof \\- list open files\n".to_string()),
    ];
    let idx = build_index(pkgs, files).unwrap();
    assert_eq!(idx.entries.len(), 2);
    assert!(idx.get("ls(1)").is_some());
    assert!(idx.get("lsof(8)").is_some());

    let hits = search(&idx, "ls");
    assert_eq!(hits[0].name, "ls(1)");
    let hits = search(&idx, "open files");
    assert!(hits.iter().any(|h| h.package == "lsof"));
}

#[test]
fn test_parse_man_variants() {
    assert_eq!(parse_man_filename("bash.1.gz").unwrap().name, "bash");
    assert_eq!(parse_man_filename("svgalib.7").unwrap().section.as_str(), "7");
}
