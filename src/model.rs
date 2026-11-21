//! Index model: packages → man page content entries.
//!
//! Mirrors upstream `Package`/`Content` Django models: packages provide
//! man pages via their files-db entries under `usr/share/man/`.

use std::collections::BTreeMap;
use std::path::Path;

use crate::manpage::{ManPage, parse_man_filename};
use crate::ManwebError;

/// A package that ships man pages (subset of upstream Package model).
#[derive(Debug, Clone)]
pub struct Package {
    /// Repository (`core`, `extra`, ...).
    pub repo: String,
    /// Package name.
    pub name: String,
    /// Version.
    pub version: String,
    /// Architecture.
    pub arch: String,
    /// Description.
    pub description: String,
}

/// A man page content entry (upstream `Content` model).
#[derive(Debug, Clone)]
pub struct ContentEntry {
    /// The man page metadata.
    pub man: ManPage,
    /// Owning package name.
    pub package: String,
    /// Path inside the package (`usr/share/man/man1/ls.1.gz`).
    pub file_path: String,
    /// Raw man page text (gz-decoded, uncompressed).
    pub raw: String,
    /// Extracted description (first ~200 chars of text).
    pub description: String,
}

/// The full manweb index.
#[derive(Debug, Default)]
pub struct Index {
    /// Packages that ship man pages.
    pub packages: Vec<Package>,
    /// Man page entries keyed by display name.
    pub entries: BTreeMap<String, Vec<ContentEntry>>,
}

impl Index {
    /// Look up entries by display name (`ls(1)`).
    pub fn get(&self, display: &str) -> Option<&Vec<ContentEntry>> {
        self.entries.get(display)
    }

    /// All man page names.
    pub fn names(&self) -> Vec<&String> {
        self.entries.keys().collect()
    }
}

/// Extract a short description from raw man page text.
///
/// Upstream `extract_description` pulls the first sentence/line of the
/// man page body (skipping `.TH`/`.SH` macros).
pub fn extract_description(raw: &str) -> String {
    for line in raw.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('.') || t.starts_with("'") {
            continue;
        }
        // Take up to first period or 200 chars.
        let desc = if let Some(pos) = t.find('.') {
            &t[..pos + 1]
        } else {
            t
        };
        return desc.chars().take(200).collect();
    }
    String::new()
}

/// Register a man page file path under a package in the index.
///
/// `file_path` is like `usr/share/man/man1/ls.1.gz`; returns the created
/// entry or None if the path isn't a man page.
pub fn index_man_file(
    idx: &mut Index,
    pkg: &Package,
    file_path: &str,
    raw: &str,
) -> Result<Option<ContentEntry>, ManwebError> {
    // Only index files under usr/share/man/manN/.
    let Some(man_dir_pos) = file_path.find("/man/") else {
        return Ok(None);
    };
    let after_man = &file_path[man_dir_pos + 5..];
    let Some(sep) = after_man.find('/') else {
        return Ok(None);
    };
    let section_dir = &after_man[..sep]; // e.g. "man1"
    if !section_dir.starts_with("man") {
        return Ok(None);
    }
    let filename = &after_man[sep + 1..];
    let man = parse_man_filename(filename)?;
    let entry = ContentEntry {
        description: extract_description(raw),
        man,
        package: pkg.name.clone(),
        file_path: file_path.to_string(),
        raw: raw.to_string(),
    };
    let key = entry.man.display();
    idx.entries.entry(key).or_default().push(entry.clone());
    Ok(Some(entry))
}

/// Build an index from packages + (pkgname, file_path, raw) triples.
pub fn build_index(
    packages: Vec<Package>,
    files: Vec<(String, String, String)>, // (pkgname, path, raw)
) -> Result<Index, ManwebError> {
    let mut idx = Index { packages, entries: BTreeMap::new() };
    // Map pkgname → Package.
    let pkg_map: BTreeMap<String, Package> = idx
        .packages
        .iter()
        .map(|p| (p.name.clone(), p.clone()))
        .collect();
    for (pkgname, path, raw) in files {
        if let Some(pkg) = pkg_map.get(&pkgname) {
            index_man_file(&mut idx, pkg, &path, &raw)?;
        }
    }
    Ok(idx)
}

/// Convenience: scan a directory tree for man pages (for fixtures/tests).
pub fn scan_man_dir(root: &Path) -> Result<Vec<(String, String, String)>, ManwebError> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for e in std::fs::read_dir(&dir)? {
            let e = e?;
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.is_file() {
                let fname = e.file_name().to_string_lossy().to_string();
                if fname.contains('.') && !fname.ends_with(".so") {
                    let rel = p.strip_prefix(root).unwrap_or(&p).to_string_lossy().to_string();
                    let raw = read_man_file(&p)?;
                    out.push((rel, fname, raw));
                }
            }
        }
    }
    Ok(out)
}

fn read_man_file(path: &Path) -> Result<String, ManwebError> {
    let data = std::fs::read(path)?;
    let text = if path.extension().and_then(|e| e.to_str()) == Some("gz") {
        use std::io::Read;
        let mut out = String::new();
        flate2::read::GzDecoder::new(data.as_slice())
            .read_to_string(&mut out)
            .map_err(|e| ManwebError::Db(format!("gunzip: {e}")))?;
        out
    } else {
        String::from_utf8_lossy(&data).to_string()
    };
    Ok(text)
}
