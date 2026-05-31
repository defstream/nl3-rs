//! The triple data structures.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// One end of a triple — a subject or an object.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Term {
    /// The entity type, e.g. `user` or `message` (JS `type`).
    pub ty: Option<String>,
    /// The entity value, e.g. `bob` or `42`.
    pub value: Option<String>,
}

/// The predicate (relation) of a triple.
///
/// `ty` is always `None`: the original `lib/predicate/type.js` is an
/// unimplemented stub that returns `undefined`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Predicate {
    /// Predicate type — always `None` (not implemented upstream).
    pub ty: Option<String>,
    /// The mapped predicate, e.g. `message` or `follow`.
    pub value: Option<String>,
}

/// A parsed Subject–Predicate–Object triple.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Triple {
    /// The subject — the entity the statement is about.
    pub subject: Term,
    /// The predicate — the relation connecting subject and object.
    pub predicate: Predicate,
    /// The object — the entity the subject relates to.
    pub object: Term,
}
