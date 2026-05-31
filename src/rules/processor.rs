//! Triple validation.

use super::{Rules, flip, infer_types};
use crate::Ambiguity;
use crate::classify::Classification;
use crate::error::ParseError;
use crate::triple::Triple;

/// Validate (and if necessary flip) a triple against the rules. Extends
/// `processRules`: missing entity types are first inferred from the grammar, then
/// a valid triple passes through, an only-reversed triple is flipped, and an
/// invalid triple is an error.
pub(crate) fn process(
    triple: Triple,
    classification: &Classification,
    rules: &Rules,
    ambiguity: Ambiguity,
) -> Result<Triple, ParseError> {
    let triple = infer_types(triple, rules, ambiguity)?;
    match flip::able(&triple, classification, rules) {
        Some(true) => Ok(flip::it(&triple, classification, rules)),
        Some(false) => Ok(triple),
        None => Err(ParseError::InvalidTriple(Box::new(triple))),
    }
}
