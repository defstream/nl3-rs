use nl3::{Nl3, ParseError, Triple};

fn client() -> Nl3 {
    Nl3::builder()
        .grammar([
            "users follow users",
            "users mention content",
            "users create messages",
            "users send messages",
            "users receive messages",
            "users message users",
        ])
        .vocabulary([
            ("follow", "follow"),
            ("stalk", "follow"),
            ("watch", "follow"),
            ("creat", "create"),
            ("made", "create"),
            ("wrote", "create"),
            ("send", "send"),
            ("sent", "send"),
            ("mail", "send"),
            ("retriev", "receive"),
            ("receiv", "receive"),
            ("reciev", "receive"),
            ("got", "receive"),
            ("messag", "message"),
            ("msg", "message"),
            ("contact", "message"),
        ])
        .build()
}

/// Assert a parsed triple equals the expected (type, value) shape.
fn assert_triple(
    triple: &Triple,
    subject: (Option<&str>, Option<&str>),
    predicate: Option<&str>,
    object: (Option<&str>, Option<&str>),
) {
    assert_eq!(triple.subject.ty.as_deref(), subject.0, "subject.type");
    assert_eq!(triple.subject.value.as_deref(), subject.1, "subject.value");
    assert_eq!(
        triple.predicate.value.as_deref(),
        predicate,
        "predicate.value"
    );
    assert_eq!(triple.predicate.ty, None, "predicate.type");
    assert_eq!(triple.object.ty.as_deref(), object.0, "object.type");
    assert_eq!(triple.object.value.as_deref(), object.1, "object.value");
}

#[test]
fn users_following_users() {
    let nl3 = client();
    let queries = [
        "users that follow user 42",
        "users who follow user 42",
        "users following user 42",
        "users followed user 42",
        "users which follow user 42",
        "users stalking user 42",
        "users who stalk user 42",
        "users which stalk user 42",
        "users watching user 42",
        "users who watch user 42",
        "users followed by user 42",
    ];
    for q in queries {
        let triple = nl3.parse(q).unwrap_or_else(|e| panic!("{q:?} failed: {e}"));
        assert_triple(
            &triple,
            (Some("user"), None),
            Some("follow"),
            (Some("user"), Some("42")),
        );
    }
}

#[test]
fn users_create_messages() {
    let nl3 = client();
    for q in [
        "user bob creates message 42",
        "user bob created message 42",
        "user bob wrote message 42",
        "user bob made message 42",
    ] {
        let triple = nl3.parse(q).unwrap_or_else(|e| panic!("{q:?} failed: {e}"));
        assert_triple(
            &triple,
            (Some("user"), Some("bob")),
            Some("create"),
            (Some("message"), Some("42")),
        );
    }
}

#[test]
fn users_send_messages() {
    let nl3 = client();
    for q in [
        "user bob sent message 42",
        "user bob sends message 42",
        "user bob mailed message 42",
        "user bob sended message 42",
    ] {
        let triple = nl3.parse(q).unwrap_or_else(|e| panic!("{q:?} failed: {e}"));
        assert_triple(
            &triple,
            (Some("user"), Some("bob")),
            Some("send"),
            (Some("message"), Some("42")),
        );
    }
}

#[test]
fn user_receives_message() {
    let nl3 = client();
    for q in [
        "user bob got message 42",
        "user bob received message 42",
        "user bob retrieved message 42",
        "user bob recieved message 42",
    ] {
        let triple = nl3.parse(q).unwrap_or_else(|e| panic!("{q:?} failed: {e}"));
        assert_triple(
            &triple,
            (Some("user"), Some("bob")),
            Some("receive"),
            (Some("message"), Some("42")),
        );
    }
}

#[test]
fn user_messages_user() {
    let nl3 = client();
    for q in [
        "user bob msg user jill",
        "user bob msgs user jill",
        "user bob messaged user jill",
        "user bob contacted user jill",
        "user bob contacts user jill",
    ] {
        let triple = nl3.parse(q).unwrap_or_else(|e| panic!("{q:?} failed: {e}"));
        assert_triple(
            &triple,
            (Some("user"), Some("bob")),
            Some("message"),
            (Some("user"), Some("jill")),
        );
    }
}

#[test]
fn invalid_input_is_rejected() {
    let nl3 = client();
    assert_eq!(nl3.parse(""), Err(ParseError::EmptyInput));
    assert_eq!(nl3.parse(" "), Err(ParseError::EmptyInput));
    assert_eq!(nl3.parse("       "), Err(ParseError::EmptyInput));
}

#[test]
fn invalid_triples_are_rejected() {
    let nl3 = client();
    // These phrases contain no word that maps to a known predicate, so no type
    // can be inferred and the triple is invalid.
    for q in ["dog jim hates cat sue", "monkey a jumped on bed b"] {
        assert!(
            matches!(nl3.parse(q), Err(ParseError::InvalidTriple(_))),
            "{q:?} should be an invalid triple"
        );
    }
}

#[test]
fn valid_triple_passes() {
    let nl3 = client();
    assert!(nl3.parse("user Aaron messaged user Micah").is_ok());
}
