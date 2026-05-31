use nl3::{Ambiguity, Nl3, ParseError};

fn messenger() -> Nl3 {
    Nl3::builder()
        .grammar(["users message users"])
        .vocabulary([
            ("msg", "message"),
            ("messag", "message"),
            ("contact", "message"),
        ])
        .build()
}

#[test]
fn infers_both_types_from_a_unique_predicate() {
    let nl3 = messenger();
    let t = nl3.parse("jack contacts jill").unwrap();
    assert_eq!(t.subject.ty.as_deref(), Some("user"));
    assert_eq!(t.subject.value.as_deref(), Some("jack"));
    assert_eq!(t.predicate.value.as_deref(), Some("message"));
    assert_eq!(t.object.ty.as_deref(), Some("user"));
    assert_eq!(t.object.value.as_deref(), Some("jill"));
}

#[test]
fn infers_a_single_missing_type() {
    let nl3 = messenger();
    // Subject type spelled out, object type inferred.
    let t = nl3.parse("user jack contacts jill").unwrap();
    assert_eq!(t.subject.ty.as_deref(), Some("user"));
    assert_eq!(t.object.ty.as_deref(), Some("user"));
    assert_eq!(t.object.value.as_deref(), Some("jill"));
}

#[test]
fn unique_inference_works_under_both_policies() {
    for policy in [Ambiguity::FirstMatch, Ambiguity::Error] {
        let nl3 = Nl3::builder()
            .grammar(["users message users"])
            .vocabulary([("contact", "message")])
            .ambiguity(policy)
            .build();
        // Only one candidate type, so the policy never comes into play.
        let t = nl3.parse("jack contacts jill").unwrap();
        assert_eq!(t.subject.ty.as_deref(), Some("user"));
        assert_eq!(t.object.ty.as_deref(), Some("user"));
    }
}

/// A grammar where the predicate `message` maps to two subject types.
fn ambiguous_builder() -> nl3::Nl3Builder {
    Nl3::builder()
        .grammar(["users message users", "admins message users"])
        .vocabulary([("contact", "message")])
}

#[test]
fn ambiguity_first_match_picks_declaration_order() {
    let nl3 = ambiguous_builder().ambiguity(Ambiguity::FirstMatch).build();
    let t = nl3.parse("jack contacts jill").unwrap();
    // "users message users" is declared first, so the subject is a user.
    assert_eq!(t.subject.ty.as_deref(), Some("user"));
    assert_eq!(t.object.ty.as_deref(), Some("user"));
}

#[test]
fn ambiguity_error_reports_candidates() {
    let nl3 = ambiguous_builder().ambiguity(Ambiguity::Error).build();
    match nl3.parse("jack contacts jill") {
        Err(ParseError::AmbiguousType {
            predicate,
            candidates,
        }) => {
            assert_eq!(predicate, "message");
            assert_eq!(candidates, vec!["user".to_string(), "admin".to_string()]);
        }
        other => panic!("expected AmbiguousType, got {other:?}"),
    }
}

#[test]
fn first_match_is_the_default_policy() {
    // No .ambiguity() call — defaults to FirstMatch, so this resolves.
    let nl3 = ambiguous_builder().build();
    assert!(nl3.parse("jack contacts jill").is_ok());
}

#[test]
fn inference_only_fills_missing_types_never_overrides() {
    // `create` requires its object to be a `message`. Here the object type is
    // stated explicitly as `user`, which violates the grammar. Inference fills
    // only *missing* types, so it can't rescue this — it stays invalid.
    let nl3 = Nl3::builder()
        .grammar(["users message users", "users create messages"])
        .vocabulary([("contact", "message"), ("creat", "create")])
        .build();
    assert!(matches!(
        nl3.parse("user bob creates user jill"),
        Err(ParseError::InvalidTriple(_)),
    ));
}
