#![forbid(unsafe_code)]
#![warn(missing_docs)]
//! # nl3
//!
//! Natural language triples — parse short plain-English Subject–Predicate–Object
//! phrases into validated triples.
//!
//! You supply a **grammar** (the valid S-P-O relations) and a **vocabulary**
//! (word-stem → predicate mappings), then [`Nl3::parse`] turns phrases into
//! [`Triple`]s, validating them and flipping reversed phrasings.
//!
//! ```
//! use nl3::Nl3;
//!
//! let nl3 = Nl3::builder()
//!     .grammar(["users message users"])
//!     .vocabulary([("contact", "message"), ("msg", "message")])
//!     .build();
//!
//! let triple = nl3.parse("user jack contacts user jill").unwrap();
//! assert_eq!(triple.subject.ty.as_deref(), Some("user"));
//! assert_eq!(triple.subject.value.as_deref(), Some("jack"));
//! assert_eq!(triple.predicate.value.as_deref(), Some("message"));
//! assert_eq!(triple.object.ty.as_deref(), Some("user"));
//! assert_eq!(triple.object.value.as_deref(), Some("jill"));
//! ```

mod classify;
mod error;
mod rules;
pub mod tagger;
pub mod text;
mod triple;

use std::collections::HashMap;

pub use error::ParseError;
pub use tagger::{LexiconTagger, Tagger};
pub use triple::{Predicate, Term, Triple};

use classify::classify;
use rules::{Rules, build_triple, process};

/// How to resolve an entity type that the grammar leaves ambiguous.
///
/// When a phrase omits a type (e.g. `"jack contacts jill"`), nl3 infers it from
/// the matched predicate. If the predicate maps to a single type, that type is
/// always used. This policy only applies when *more than one* candidate type
/// exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Ambiguity {
    /// Use the first candidate type, in grammar declaration order. (Default.)
    #[default]
    FirstMatch,
    /// Return [`ParseError::AmbiguousType`] instead of guessing.
    Error,
}

/// A configured nl3 client. Create one with [`Nl3::builder`]. Ports the instance
/// returned by the `create` factory in `index.js`.
pub struct Nl3 {
    rules: Rules,
    tagger: Box<dyn Tagger>,
    ambiguity: Ambiguity,
}

impl Nl3 {
    /// Start building a client.
    pub fn builder() -> Nl3Builder {
        Nl3Builder::default()
    }

    /// Parse a phrase into a [`Triple`]. Ports `lib/parse.js`.
    ///
    /// # Errors
    /// Returns [`ParseError::EmptyInput`] for empty/whitespace input, and
    /// [`ParseError::InvalidTriple`] when the phrase does not match the grammar
    /// in either direction.
    pub fn parse(&self, text: &str) -> Result<Triple, ParseError> {
        if text.trim().is_empty() {
            return Err(ParseError::EmptyInput);
        }
        let classification = classify(text, self.tagger.as_ref());
        let triple = build_triple(&classification, &self.rules);
        process(triple, &classification, &self.rules, self.ambiguity)
    }
}

/// Builder for [`Nl3`].
#[derive(Default)]
pub struct Nl3Builder {
    grammar: Vec<String>,
    vocabulary: HashMap<String, String>,
    tagger: Option<Box<dyn Tagger>>,
    ambiguity: Ambiguity,
}

impl Nl3Builder {
    /// Set the grammar: valid triples written as `"Subject Predicate Object"`
    /// (all words are singularized). Replaces any previously set grammar.
    pub fn grammar<I, S>(mut self, grammar: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.grammar = grammar.into_iter().map(Into::into).collect();
        self
    }

    /// Set the vocabulary: a mapping of word stems to predicates within the
    /// grammar. Replaces any previously set vocabulary.
    pub fn vocabulary<I, K, V>(mut self, vocabulary: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.vocabulary = vocabulary
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        self
    }

    /// Override the part-of-speech tagger. Defaults to [`LexiconTagger`].
    pub fn tagger(mut self, tagger: impl Tagger + 'static) -> Self {
        self.tagger = Some(Box::new(tagger));
        self
    }

    /// Set how ambiguous inferred types are resolved. Defaults to
    /// [`Ambiguity::FirstMatch`].
    pub fn ambiguity(mut self, ambiguity: Ambiguity) -> Self {
        self.ambiguity = ambiguity;
        self
    }

    /// Build the client.
    pub fn build(self) -> Nl3 {
        Nl3 {
            rules: Rules::build(&self.grammar, self.vocabulary),
            tagger: self.tagger.unwrap_or_else(|| Box::new(LexiconTagger)),
            ambiguity: self.ambiguity,
        }
    }
}
