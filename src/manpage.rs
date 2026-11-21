//! Man page model + filename parsing.
//!
//! Upstream filenames follow `name.section[.language].gz`, e.g.
//! `ls.1.gz`, `bash.1.gz`, `intro.5.gz`. Sections: 1=user cmds,
//! 2=syscalls, 3=lib funcs, 4=devices, 5=file formats, 6=games,
//! 7=misc, 8=sysadmin, 9=kernel, n= Tcl, l=lib, o=other.

use crate::ManwebError;

/// Man section number (1-9 or letter).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ManSection {
    /// Numeric section.
    Num(u8),
    /// Letter section (n, l, o).
    Letter(char),
}

impl ManSection {
    /// Parse section char (digit or n/l/o).
    pub fn parse(c: char) -> Option<Self> {
        if c.is_ascii_digit() {
            Some(Self::Num(c as u8 - b'0'))
        } else if matches!(c, 'n' | 'l' | 'o') {
            Some(Self::Letter(c))
        } else {
            None
        }
    }

    /// Display form.
    pub fn as_str(&self) -> String {
        match self {
            Self::Num(n) => n.to_string(),
            Self::Letter(c) => c.to_string(),
        }
    }
}

/// A parsed man page filename.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManPage {
    /// Command/function name (e.g. `ls`, `intro`).
    pub name: String,
    /// Section.
    pub section: ManSection,
    /// Language code if present (e.g. `fr`, `de`).
    pub lang: Option<String>,
    /// Compression suffix (`gz`, `xz`, `zst`, `bz2`, or none).
    pub compression: Option<String>,
}

impl ManPage {
    /// Canonical display name `name(section)`.
    pub fn display(&self) -> String {
        format!("{}({})", self.name, self.section.as_str())
    }

    /// URL-safe path used by archmanweb: `/man/<name>.<section>`.
    pub fn url_path(&self) -> String {
        format!("/man/{}.{}", self.name, self.section.as_str())
    }
}

/// Parse a man page filename like `ls.1.gz` or `intro.fr.5` → [`ManPage`].
pub fn parse_man_filename(filename: &str) -> Result<ManPage, ManwebError> {
    let stem = filename
        .strip_suffix(".gz")
        .or_else(|| filename.strip_suffix(".xz"))
        .or_else(|| filename.strip_suffix(".zst"))
        .or_else(|| filename.strip_suffix(".bz2"))
        .unwrap_or(filename);
    let compression = if stem.len() != filename.len() {
        Some(filename[stem.len() + 1..].to_string())
    } else {
        None
    };
    // stem like "name.1", "name.fr.1", "name.1p"? Handle name.lang.section.
    let parts: Vec<&str> = stem.rsplitn(3, '.').collect();
    // parts[0]=section(+lang?), parts[1]=lang?, parts[2]=name
    let section_part = parts[0];
    let (name, lang) = if parts.len() == 3 {
        (parts[2], Some(parts[1].to_string()))
    } else if parts.len() == 2 {
        (parts[1], None)
    } else {
        return Err(ManwebError::InvalidManPage(filename.to_string()));
    };
    // section must be a single valid char (1-9, n, l, o).
    let mut chars = section_part.chars();
    let sec_char = chars.next()
        .ok_or_else(|| ManwebError::InvalidManPage(filename.to_string()))?;
    if chars.next().is_some() {
        return Err(ManwebError::InvalidManPage(filename.to_string()));
    }
    let section = ManSection::parse(sec_char)
        .ok_or_else(|| ManwebError::InvalidManPage(filename.to_string()))?;
    Ok(ManPage {
        name: name.to_string(),
        section,
        lang,
        compression,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_common() {
        let p = parse_man_filename("ls.1.gz").unwrap();
        assert_eq!(p.name, "ls");
        assert_eq!(p.section, ManSection::Num(1));
        assert_eq!(p.compression.as_deref(), Some("gz"));
        assert_eq!(p.url_path(), "/man/ls.1");
    }

    #[test]
    fn test_parse_lang() {
        let p = parse_man_filename("intro.fr.5").unwrap();
        assert_eq!(p.name, "intro");
        assert_eq!(p.lang.as_deref(), Some("fr"));
    }

    #[test]
    fn test_parse_letter_section() {
        let p = parse_man_filename("tclsh.n.gz").unwrap();
        assert_eq!(p.section, ManSection::Letter('n'));
    }

    #[test]
    fn test_invalid() {
        assert!(parse_man_filename("README").is_err());
        assert!(parse_man_filename("x.9z.gz").is_err());
    }
}
