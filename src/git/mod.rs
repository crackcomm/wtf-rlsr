//! Git wrappers and helpers module.

mod commit;
mod diff;
pub mod pull;
mod repository;

pub use self::commit::*;
pub use self::diff::*;
pub use self::repository::*;

/// WTF RLS latest git commit tag.
pub const WTF_RLSR_TAG: &'static str = ":refs/tags/wtf-rlsr-latest";
