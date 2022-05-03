#![doc(html_root_url = "https://docs.rs/miniserde/0.1.24")]
#![allow(
    clippy::needless_doctest_main,
    clippy::vec_init_then_push,
    // Regression causing false positives:
    // https://github.com/rust-lang/rust-clippy/issues/5343
    clippy::useless_transmute,
    // Clippy bug: https://github.com/rust-lang/rust-clippy/issues/5704
    clippy::unnested_or_patterns,
    // We support older compilers.
    clippy::manual_range_contains,
    // Pedantic.
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::checked_conversions,
    clippy::doc_markdown,
    clippy::enum_glob_use,
    clippy::let_underscore_drop,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::redundant_else,
    clippy::shadow_unrelated,
    clippy::single_match_else,
    clippy::too_many_lines,
)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[doc(hidden)]
pub use mini_internal::*;

// These derives were renamed from MiniTrait -> Trait with the release of Rust
// 1.30.0. Keep exposing the old names for backward compatibility but remove in
// the next major version of Miniserde.
#[doc(hidden)]
pub use mini_internal::Deserialize as MiniDeserialize;

// Not public API.
#[doc(hidden)]
#[path = "export.rs"]
pub mod __private;

#[macro_use]
mod careful;

#[macro_use]
mod place;

mod error;
mod ignore;
mod ptr;

pub mod de;
pub mod json;

#[doc(inline)]
pub use crate::de::Deserialize;
pub use crate::error::{Error, Result};

make_place!(Place);
