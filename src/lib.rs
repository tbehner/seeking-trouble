//! Extract source code that was changed by a commit containing certain keywords.
//!
//! ```rust
//! use seeking_trouble::code_repository::CodeRepository;
//! use regex::Regex;
//! let repository = CodeRepository::new(".").unwrap();
//! let patterns = vec![Regex::new("bug").unwrap()];
//! for oid in repository.commits_matching(&patterns).unwrap() {
//!     repository.get_changes(oid);
//! }
//! ```

pub mod code_repository;
pub mod code_region;
pub mod change_set;
