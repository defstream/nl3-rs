//! Serialization round-trip tests. Only built with the `serde` feature:
//! `cargo test --features serde`.
#![cfg(feature = "serde")]

use nl3::{Nl3, Triple};

fn client() -> Nl3 {
    Nl3::builder()
        .grammar(["users message users"])
        .vocabulary([("contact", "message")])
        .build()
}

#[test]
fn triple_round_trips_through_json() {
    let triple = client().parse("user jack contacts user jill").unwrap();

    let json = serde_json::to_string(&triple).unwrap();
    let back: Triple = serde_json::from_str(&json).unwrap();

    assert_eq!(triple, back);
}

#[test]
fn triple_serializes_to_expected_shape() {
    let triple = client().parse("user jack contacts user jill").unwrap();
    let value: serde_json::Value = serde_json::to_value(&triple).unwrap();

    assert_eq!(value["subject"]["ty"], "user");
    assert_eq!(value["subject"]["value"], "jack");
    assert_eq!(value["predicate"]["value"], "message");
    assert_eq!(value["object"]["ty"], "user");
    assert_eq!(value["object"]["value"], "jill");
}
