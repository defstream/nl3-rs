//! The smallest useful nl3 program: one grammar rule, a few vocabulary
//! synonyms, and a handful of phrasings that all parse to the same triple.
//!
//! Run with:
//!
//! ```shell
//! cargo run --example basic
//! ```

use nl3::Nl3;

fn main() {
    // A client knows one relation — "a user messages a user" — and a few
    // words that all mean "message".
    let nl3 = Nl3::builder()
        .grammar(["users message users"])
        .vocabulary([
            ("msg", "message"),     // user jack msgs user jill
            ("messag", "message"),  // user jack messaged user jill
            ("contact", "message"), // user jack contacted user jill
        ])
        .build();

    // These phrasings differ only in the verb; every one yields the same triple.
    let phrasings = [
        "user jack msg user jill",
        "user jack msgs user jill",
        "user jack messaged user jill",
        "user jack contacted user jill",
        "user jack contacts user jill",
        "jack contacts jill",
    ];

    for phrase in phrasings {
        match nl3.parse(phrase) {
            Ok(triple) => println!(
                "{phrase:<35} => {}({}) -{}-> {}({})",
                triple.subject.ty.as_deref().unwrap_or("?"),
                triple.subject.value.as_deref().unwrap_or("?"),
                triple.predicate.value.as_deref().unwrap_or("?"),
                triple.object.ty.as_deref().unwrap_or("?"),
                triple.object.value.as_deref().unwrap_or("?"),
            ),
            Err(e) => println!("{phrase:<35} => error: {e}"),
        }
    }
}
