//! SigmaOS archmanweb component.
//!
//! Rust implementation of `archlinux/archmanweb`: the Arch Linux man pages
//! repository — indexes man pages shipped by official packages and serves
//! them with search.
//!
//! Upstream is a Django app (`archmanweb/`) that reads pacman syncdb +
//! files dbs, extracts `usr/share/man/**` entries, runs `mandoc` for HTML,
//! and provides search. This port implements the indexing + search core;
//! HTML rendering stays with the caller (or `mandoc` CLI).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod manpage;
pub mod model;
pub mod search;

pub use error::ManwebError;
pub use manpage::{ManPage, ManSection, parse_man_filename};
pub use model::{ContentEntry, Index, Package, build_index};
pub use search::{SearchHit, search};
