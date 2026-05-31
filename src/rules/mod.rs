//! The triple rules engine.
mod flip;
pub(crate) mod parse;
mod processor;

use std::collections::HashMap;

use crate::Ambiguity;
use crate::classify::Classification;
use crate::error::ParseError;
use crate::text::singularize;
use crate::triple::{Predicate, Term, Triple};

pub(crate) use processor::process;

/// One grammar rule in declaration order, used for deterministic type inference.
pub(crate) struct GrammarRule {
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

/// A grammar entry keyed by subject: `predicate -> valid object types`.
#[derive(Default)]
pub(crate) struct SubjectEntry {
    pub predicates: HashMap<String, PredicateObjects>,
}

/// A grammar entry keyed by object: `predicate -> valid subject types`.
#[derive(Default)]
pub(crate) struct ObjectEntry {
    pub predicates: HashMap<String, PredicateSubjects>,
}

pub(crate) struct PredicateObjects {
    pub objects: Vec<String>,
}

pub(crate) struct PredicateSubjects {
    #[allow(dead_code)] // stored for parity with the JS rules; never read by the pipeline
    pub subjects: Vec<String>,
}

/// The compiled rules engine. Ports the object built by `rules-engine/index.js`.
pub(crate) struct Rules {
    pub subjects: HashMap<String, SubjectEntry>,
    #[allow(dead_code)] // built for parity; the pipeline matches via `subjects`/`objects`
    pub predicates: Vec<String>,
    pub objects: HashMap<String, ObjectEntry>,
    pub vocabulary: HashMap<String, String>,
    /// The parsed grammar rules in declaration order (for type inference).
    pub entries: Vec<GrammarRule>,
}

impl Rules {
    /// Build the rules engine from grammar lines and a vocabulary map. Ports
    /// `rules-engine/index.js` + `parse/rules.js`, `subjects.js`, `objects.js`,
    /// `reduce-predicates.js`.
    pub(crate) fn build(grammar: &[String], vocabulary: HashMap<String, String>) -> Rules {
        let mut rules = Rules {
            subjects: HashMap::new(),
            predicates: Vec::new(),
            objects: HashMap::new(),
            vocabulary,
            entries: Vec::new(),
        };

        for line in grammar {
            let tokens: Vec<&str> = line.split(' ').collect();
            let subject = singularize(tokens.first().copied().unwrap_or_default());
            let predicate = singularize(tokens.get(1).copied().unwrap_or_default());
            let object = singularize(tokens.get(2).copied().unwrap_or_default());

            // Record the rule in declaration order for type inference.
            rules.entries.push(GrammarRule {
                subject: subject.clone(),
                predicate: predicate.clone(),
                object: object.clone(),
            });

            // include subject -> predicate -> [object] (only if absent).
            rules
                .subjects
                .entry(subject.clone())
                .or_default()
                .predicates
                .entry(predicate.clone())
                .or_insert_with(|| PredicateObjects {
                    objects: vec![object.clone()],
                });

            // record the predicate once.
            if !rules.predicates.contains(&predicate) {
                rules.predicates.push(predicate.clone());
            }

            // include object -> predicate -> [subject] (only if absent).
            rules
                .objects
                .entry(object)
                .or_default()
                .predicates
                .entry(predicate)
                .or_insert_with(|| PredicateSubjects {
                    subjects: vec![subject.clone()],
                });
        }

        rules
    }
}

/// Build a triple from a classification. Ports `triple.js` and the
/// `subject`/`predicate`/`object` assembly modules.
pub(crate) fn build_triple(classification: &Classification, rules: &Rules) -> Triple {
    let parts = parse::parts(classification, rules);

    Triple {
        subject: Term {
            // subjectType: subjects[0] || objects[0] (from before-predicate tokens).
            ty: parts
                .subjects
                .subjects
                .first()
                .or_else(|| parts.subjects.objects.first())
                .cloned(),
            // subjectValue: first(after ++ before) values.
            value: first_value(&parts.subjects),
        },
        predicate: Predicate {
            // predicateType is an upstream stub that always returns undefined.
            ty: None,
            // predicateValue: map the predicate token through the vocabulary.
            value: parts.predicate.as_ref().and_then(|(token, _tag)| {
                if token.is_empty() {
                    None
                } else {
                    parse::map_predicate(token, &rules.vocabulary)
                }
            }),
        },
        object: Term {
            // objectType: objects[0] || subjects.subjects[0] (from before-predicate tokens).
            ty: parts
                .objects
                .objects
                .first()
                .or_else(|| parts.subjects.subjects.first())
                .cloned(),
            // objectValue: first(after ++ before) values.
            value: first_value(&parts.objects),
        },
    }
}

/// `first(reduced.after.map(first).concat(reduced.before.map(first)))` — the
/// first value token, preferring those after the matched type, then before.
fn first_value(reduced: &parse::Reduced) -> Option<String> {
    reduced
        .after
        .first()
        .or_else(|| reduced.before.first())
        .map(|(token, _tag)| token.clone())
}

/// Fill in any missing subject/object types by inferring them from the grammar
/// via the matched predicate. This lets phrases that omit the entity type — e.g.
/// `"jack contacts jill"` against `"users message users"` — still resolve, with
/// `jack` and `jill` treated as `user`s.
///
/// A type is inferred only when the predicate determines it. When more than one
/// candidate type exists, [`Ambiguity`] decides whether to take the first (in
/// grammar declaration order) or return [`ParseError::AmbiguousType`].
pub(crate) fn infer_types(
    mut triple: Triple,
    rules: &Rules,
    ambiguity: Ambiguity,
) -> Result<Triple, ParseError> {
    // Inference is driven entirely by the predicate; without one there is
    // nothing to infer from.
    let Some(predicate) = triple.predicate.value.clone() else {
        return Ok(triple);
    };

    // Subject type: any subject whose grammar rules include this predicate.
    if triple.subject.ty.is_none() {
        let candidates = ordered_dedup(
            rules
                .entries
                .iter()
                .filter(|e| e.predicate == predicate)
                .map(|e| e.subject.clone()),
        );
        triple.subject.ty = pick(&predicate, candidates, ambiguity)?;
    }

    // Object type: the objects allowed for the (now-known) subject + predicate.
    if triple.object.ty.is_none()
        && let Some(subject) = triple.subject.ty.clone()
    {
        let candidates = ordered_dedup(
            rules
                .entries
                .iter()
                .filter(|e| e.subject == subject && e.predicate == predicate)
                .map(|e| e.object.clone()),
        );
        triple.object.ty = pick(&predicate, candidates, ambiguity)?;
    }

    Ok(triple)
}

/// Collect items preserving first-seen order and dropping duplicates.
fn ordered_dedup(iter: impl Iterator<Item = String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for item in iter {
        if !out.contains(&item) {
            out.push(item);
        }
    }
    out
}

/// Choose a single type from the candidates: `None` if there are none, the only
/// one if unique, otherwise resolved per the [`Ambiguity`] policy.
fn pick(
    predicate: &str,
    mut candidates: Vec<String>,
    ambiguity: Ambiguity,
) -> Result<Option<String>, ParseError> {
    match candidates.len() {
        0 => Ok(None),
        1 => Ok(Some(candidates.remove(0))),
        _ => match ambiguity {
            Ambiguity::FirstMatch => Ok(Some(candidates.remove(0))),
            Ambiguity::Error => Err(ParseError::AmbiguousType {
                predicate: predicate.to_string(),
                candidates,
            }),
        },
    }
}
