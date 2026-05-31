//! Error types.
use std::fmt;

use crate::triple::Triple;

/// An error produced while parsing a phrase into a triple.
///
/// Marked `#[non_exhaustive]`: match with a wildcard arm so future variants do
/// not break your build.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseError {
    /// The supplied text was empty or whitespace-only and could not be parsed.
    /// Ports the `"could not be parsed into a triple"` throw in `parse.js`.
    EmptyInput,
    /// The parsed triple is not valid in either direction against the grammar.
    /// Ports the `"Invalid triple"` throw in `rules-processor.js`. The triple is
    /// boxed to keep `Result<Triple, ParseError>` small on the common `Ok` path.
    InvalidTriple(Box<Triple>),
    /// A phrase omitted an entity type, and the matched predicate maps to more
    /// than one candidate type in the grammar, so the type cannot be inferred
    /// unambiguously. Only returned under [`crate::Ambiguity::Error`].
    AmbiguousType {
        /// The predicate whose subject/object type was ambiguous.
        predicate: String,
        /// The candidate types, in grammar declaration order.
        candidates: Vec<String>,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::EmptyInput => {
                write!(f, "the supplied text could not be parsed into a triple")
            }
            ParseError::InvalidTriple(triple) => write!(f, "invalid triple: {triple:?}"),
            ParseError::AmbiguousType {
                predicate,
                candidates,
            } => write!(
                f,
                "ambiguous type for predicate {predicate:?}: candidates {candidates:?}"
            ),
        }
    }
}

impl std::error::Error for ParseError {}
