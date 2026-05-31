//! Triple flipping.

use super::Rules;
use super::parse::last_predicate;
use crate::classify::Classification;
use crate::triple::{Predicate, Triple};

/// Does `subject_ty --predicate--> object_ty` exist in the rules? Ports the
/// private `itAbides` helper.
fn it_abides(
    subject_ty: Option<&str>,
    predicate_value: Option<&str>,
    object_ty: Option<&str>,
    rules: &Rules,
) -> bool {
    let Some(subject_ty) = subject_ty else {
        return false;
    };
    let Some(subject) = rules.subjects.get(subject_ty) else {
        return false;
    };
    let Some(predicate_value) = predicate_value else {
        return false;
    };
    let Some(predicate) = subject.predicates.get(predicate_value) else {
        return false;
    };
    let Some(object_ty) = object_ty else {
        return false;
    };
    predicate.objects.iter().any(|o| o == object_ty)
}

/// Whether the triple should be flipped. Ports `flip.able`.
///
/// Returns `Some(false)` if the triple is already valid, `Some(true)` if it is
/// only valid reversed, and `None` if it is invalid in both directions.
pub(super) fn able(
    triple: &Triple,
    classification: &Classification,
    rules: &Rules,
) -> Option<bool> {
    let abides = it_abides(
        triple.subject.ty.as_deref(),
        triple.predicate.value.as_deref(),
        triple.object.ty.as_deref(),
        rules,
    );
    if abides {
        return Some(false);
    }

    // Try the reversed triple with the same predicate...
    let reversed = it_abides(
        triple.object.ty.as_deref(),
        triple.predicate.value.as_deref(),
        triple.subject.ty.as_deref(),
        rules,
    );
    // ...and reversed with the "last predicate" (always None upstream; see
    // `last_predicate`), which can therefore never abide.
    let last = last_predicate(classification, rules);
    let reversed_last = it_abides(
        triple.object.ty.as_deref(),
        last.as_deref(),
        triple.subject.ty.as_deref(),
        rules,
    );

    if reversed || reversed_last {
        Some(true)
    } else {
        None
    }
}

/// Flip a triple: swap subject and object, take the predicate from
/// `last_predicate`. Ports `flip.it`.
pub(super) fn it(triple: &Triple, classification: &Classification, rules: &Rules) -> Triple {
    Triple {
        subject: triple.object.clone(),
        predicate: Predicate {
            ty: None,
            value: last_predicate(classification, rules),
        },
        object: triple.subject.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::triple::Term;
    use std::collections::HashMap;

    fn fixture_rules() -> Rules {
        let grammar: Vec<String> = ["users message users", "users follow users"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        Rules::build(&grammar, HashMap::new())
    }

    fn empty_classification() -> Classification {
        Classification {
            parts: Vec::new(),
            text: String::new(),
        }
    }

    fn term(ty: &str, value: &str) -> Term {
        Term {
            ty: Some(ty.into()),
            value: Some(value.into()),
        }
    }

    #[test]
    fn it_swaps_subject_and_object() {
        // Ports test/unit/flip/001-flip.js: flip.it swaps subject and object.
        let triple = Triple {
            subject: term("user", "Bob"),
            predicate: Predicate {
                ty: None,
                value: Some("message".into()),
            },
            object: term("user", "Jill"),
        };
        let flipped = it(&triple, &empty_classification(), &fixture_rules());
        assert_eq!(flipped.object, triple.subject);
        assert_eq!(flipped.subject, triple.object);
        // last_predicate is the upstream no-op, so the predicate value is None.
        assert_eq!(flipped.predicate.value, None);
    }

    #[test]
    fn able_is_false_for_a_valid_triple() {
        let rules = fixture_rules();
        let triple = Triple {
            subject: term("user", "Bob"),
            predicate: Predicate {
                ty: None,
                value: Some("message".into()),
            },
            object: term("user", "Jill"),
        };
        assert_eq!(able(&triple, &empty_classification(), &rules), Some(false));
    }

    #[test]
    fn able_is_none_for_an_invalid_triple() {
        let rules = fixture_rules();
        let triple = Triple {
            subject: term("dog", "rex"),
            predicate: Predicate {
                ty: None,
                value: Some("message".into()),
            },
            object: term("cat", "tom"),
        };
        assert_eq!(able(&triple, &empty_classification(), &rules), None);
    }
}
