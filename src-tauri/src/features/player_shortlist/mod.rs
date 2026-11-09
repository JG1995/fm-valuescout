//! Player shortlists that FM exports as `.fmf` files.
//!
//! `file` reads a shortlist file into its list name and player Unique IDs.

// The reader has no caller until a save can link a shortlist file.
#![cfg_attr(not(test), allow(dead_code))]

mod file;
#[cfg(test)]
mod test_support;
