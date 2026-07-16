//! BDD test shim for pheno-plugin-vessel.
//!
//! This file uses a local vendored helper module to avoid fragile cross-worktree
//! boundaries.
mod steps_shared;
pub use steps_shared::*;
