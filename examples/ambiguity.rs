//! When a phrase omits an entity type, nl3 infers it from the grammar via the
//! matched predicate. If the predicate maps to more than one candidate type,
//! the [`Ambiguity`] policy decides what happens.
//!
//! This example uses a grammar where `message` relates BOTH `users` and
//! `admins` to `users`, then contrasts the two policies on the same bare phrase:
//!
//! - `Ambiguity::FirstMatch` — pick the first matching rule (here, `user`).
//! - `Ambiguity::Error`      — refuse to guess and return `AmbiguousType`.
//!
//! Run with:
//!
//! ```shell
//! cargo run --example ambiguity
//! ```

use nl3::{Ambiguity, Nl3, ParseError};

fn build(policy: Ambiguity) -> Nl3 {
    // `message` has two possible subject types: `user` and `admin`.
    Nl3::builder()
        .grammar(["users message users", "admins message users"])
        .vocabulary([("contact", "message"), ("msg", "message")])
        .ambiguity(policy)
        .build()
}

fn main() {
    // The type is spelled out, so inference never runs — both policies agree.
    let explicit = "admin alice contacts user bob";
    // The subject type is omitted, so it must be inferred — this is ambiguous.
    let bare = "alice contacts bob";

    println!("== Ambiguity::FirstMatch (default) ==");
    let first = build(Ambiguity::FirstMatch);
    report(&first, explicit);
    report(&first, bare);

    println!("\n== Ambiguity::Error ==");
    let strict = build(Ambiguity::Error);
    report(&strict, explicit);
    report(&strict, bare);
}

fn report(nl3: &Nl3, phrase: &str) {
    match nl3.parse(phrase) {
        Ok(t) => println!(
            "  ok    {phrase:<32} => {}({}) -{}-> {}({})",
            t.subject.ty.as_deref().unwrap_or("?"),
            t.subject.value.as_deref().unwrap_or("-"),
            t.predicate.value.as_deref().unwrap_or("?"),
            t.object.ty.as_deref().unwrap_or("?"),
            t.object.value.as_deref().unwrap_or("-"),
        ),
        Err(ParseError::AmbiguousType {
            predicate,
            candidates,
        }) => {
            println!("  error {phrase:<32} => ambiguous {predicate:?}: pick one of {candidates:?}")
        }
        Err(e) => println!("  error {phrase:<32} => {e}"),
    }
}
