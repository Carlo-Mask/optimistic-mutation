// Inspired by: https://www.roc-lang.org/functional#opportunistic-mutation

pub mod rc;
pub mod sync;
pub mod to_owned;

#[cfg(feature = "serde")]
pub mod serde;
