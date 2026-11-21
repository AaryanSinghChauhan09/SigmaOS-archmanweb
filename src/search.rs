//! Search over the manweb index — apropos-style + name search.
//!
//! Upstream uses Postgres `to_tsvector` full-text search; this port
//! implements a deterministic in-memory scorer: name matches rank above
//! description matches, with simple term-frequency weighting.

use crate::model::Index;

/// A search result.
#[derive(Debug, Clone)]
pub struct SearchHit {
    /// Man page display name (`ls(1)`).
    pub name: String,
    /// Owning package.
    pub package: String,
    /// Description snippet.
    pub description: String,
    /// Relevance score (higher = better).
    pub score: u32,
}

/// Search the index by query terms.
///
/// `query` is whitespace-separated terms; all terms must appear in either
/// the man page name or description (AND semantics, like upstream search).
pub fn search(idx: &Index, query: &str) -> Vec<SearchHit> {
    let terms: Vec<String> = query
        .split_whitespace()
        .map(|t| t.to_lowercase())
        .filter(|t| !t.is_empty())
        .collect();
    if terms.is_empty() {
        return Vec::new();
    }
    let mut hits = Vec::new();
    for (name, entries) in &idx.entries {
        for e in entries {
            let name_lower = name.to_lowercase();
            let desc_lower = e.description.to_lowercase();
            let name_match = terms.iter().all(|t| name_lower.contains(t.as_str()));
            let desc_match = terms.iter().all(|t| desc_lower.contains(t.as_str()));
            if !name_match && !desc_match {
                continue;
            }
            let mut score = 0u32;
            if name_match {
                score += 100;
                // Exact prefix match bonus.
                let bare = name_lower.trim_end_matches(|c: char| c.is_ascii_digit() || c == '(');
                if terms.iter().any(|t| bare.starts_with(t.as_str())) {
                    score += 50;
                }
            }
            if desc_match {
                score += 10;
                for t in &terms {
                    score += desc_lower.matches(t.as_str()).count() as u32;
                }
            }
            hits.push(SearchHit {
                name: name.clone(),
                package: e.package.clone(),
                description: e.description.clone(),
                score,
            });
        }
    }
    hits.sort_by(|a, b| b.score.cmp(&a.score).then(a.name.cmp(&b.name)));
    hits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ContentEntry, Index, Package};
    use crate::manpage::{ManPage, ManSection};

    fn idx_with(entries: Vec<(&str, &str, &str)>) -> Index {
        let mut idx = Index::default();
        for (name, pkg, desc) in entries {
            idx.packages.push(Package {
                repo: "extra".into(), name: pkg.into(), version: "1".into(),
                arch: "x86_64".into(), description: String::new(),
            });
            let man = ManPage {
                name: name.into(),
                section: ManSection::Num(1),
                lang: None,
                compression: None,
            };
            let key = man.display();
            idx.entries.entry(key).or_default().push(ContentEntry {
                man, package: pkg.into(), file_path: String::new(),
                raw: String::new(), description: desc.into(),
            });
        }
        idx
    }

    #[test]
    fn test_name_search_ranks_first() {
        let idx = idx_with(vec![
            ("ls", "coreutils", "list directory contents"),
            ("lsof", "lsof", "list open files"),
        ]);
        let hits = search(&idx, "ls");
        assert!(!hits.is_empty());
        // "ls(1)" name match should outrank description matches.
        assert_eq!(hits[0].name, "ls(1)");
    }

    #[test]
    fn test_description_search() {
        let idx = idx_with(vec![
            ("foo", "foo", "a tool for frobbing"),
        ]);
        let hits = search(&idx, "frobbing");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].package, "foo");
    }

    #[test]
    fn test_empty_query() {
        let idx = idx_with(vec![]);
        assert!(search(&idx, "").is_empty());
    }
}
