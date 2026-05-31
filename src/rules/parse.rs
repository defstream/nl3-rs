//! Parsing of a classification into the raw parts of a triple.

use crate::classify::Classification;
use crate::rules::Rules;
use crate::text::{singularize, stem_key};

/// A predicate located within the token stream. Ports the `{index, value}`
/// object from `first-predicate.js`.
pub(crate) struct PredicateHit {
    pub index: usize,
    #[allow(dead_code)] // mirrors the JS shape; `value` is recomputed downstream
    pub value: String,
}

/// Tokens bucketed by role, the output of `reduce-parts.js`.
#[derive(Default)]
pub(crate) struct Reduced {
    pub objects: Vec<String>,
    pub subjects: Vec<String>,
    pub before: Vec<(String, String)>,
    pub after: Vec<(String, String)>,
}

/// The parts of a phrase, split around its predicate. Ports `parts.js`.
pub(crate) struct Parts {
    /// `reduce_parts` of the tokens after the predicate.
    pub objects: Reduced,
    /// `reduce_parts` of the tokens before the predicate.
    pub subjects: Reduced,
    /// The `(token, tag)` pair identified as the predicate, if any.
    pub predicate: Option<(String, String)>,
}

/// Map a word to a predicate via the vocabulary. Ports `parse/predicate.js`:
/// `vocabulary[stemmer(singularize(text))]`.
pub(crate) fn map_predicate(
    text: &str,
    vocabulary: &std::collections::HashMap<String, String>,
) -> Option<String> {
    let key = stem_key(text);
    vocabulary.get(&key).cloned()
}

/// Find the first token that maps to a predicate. Ports `first-predicate.js`.
pub(crate) fn first_predicate(
    classification: &Classification,
    rules: &Rules,
) -> Option<PredicateHit> {
    for (index, (token, _tag)) in classification.parts.iter().enumerate() {
        if let Some(value) = map_predicate(token, &rules.vocabulary) {
            return Some(PredicateHit { index, value });
        }
    }
    None
}

/// Find the *last* token that maps to a predicate. Ports `last-predicate.js`.
///
/// NOTE: This faithfully reproduces an upstream bug. The original loop is
/// `for (i = tokens.length - 1; i < -1; i = i - 1)`, whose condition `i < -1`
/// is false on entry, so the body never runs and the function always returns
/// `undefined`. The flip logic depends on this behavior, so we preserve it by
/// always returning `None` rather than "fixing" the iteration.
pub(crate) fn last_predicate(_classification: &Classification, _rules: &Rules) -> Option<String> {
    None
}

/// Bucket tokens into subjects/objects (known from the grammar) and otherwise
/// before/after values. Ports `reduce-parts.js`.
pub(crate) fn reduce_parts(data: &[(String, String)], rules: &Rules) -> Reduced {
    let mut result = Reduced::default();
    for part in data {
        let normalized = singularize(&part.0);
        let mut added = false;
        if rules.objects.contains_key(&normalized) {
            result.objects.push(normalized.clone());
            added = true;
        }
        if rules.subjects.contains_key(&normalized) {
            result.subjects.push(normalized.clone());
            added = true;
        }
        // Skip prepositions (IN) and WH-words (tag starting with `W`).
        if !added && part.1 != "IN" && !part.1.starts_with('W') {
            if result.subjects.is_empty() && result.objects.is_empty() {
                result.before.push(part.clone());
            } else {
                result.after.push(part.clone());
            }
        }
    }
    result
}

/// Split a classification into its parts around the first predicate. Ports
/// `parts.js` (the unused `prepositionIndex` computation is dropped).
pub(crate) fn parts(classification: &Classification, rules: &Rules) -> Parts {
    let predicate = first_predicate(classification, rules);

    let (before, after, predicate_part) = match &predicate {
        // JS `index && slice(...)`: a predicate at index 0 is falsy, so the
        // before/after slices are empty, but the predicate token is still set.
        Some(hit) if hit.index > 0 => (
            classification.parts[..hit.index].to_vec(),
            classification.parts[hit.index + 1..].to_vec(),
            Some(classification.parts[hit.index].clone()),
        ),
        Some(hit) => (
            Vec::new(),
            Vec::new(),
            Some(classification.parts[hit.index].clone()),
        ),
        None => (Vec::new(), Vec::new(), None),
    };

    Parts {
        objects: reduce_parts(&after, rules),
        subjects: reduce_parts(&before, rules),
        predicate: predicate_part,
    }
}
