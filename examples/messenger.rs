//! A fuller example modeled on a social-messenger domain: users follow each
//! other, mention content, and create/send/receive/message. This mirrors the
//! grammar and vocabulary from the original nl3 test suite.
//!
//! It also shows the two failure modes — empty input and a triple that does not
//! fit the grammar — and how nl3 *flips* a reversed phrasing back into a valid
//! triple.
//!
//! Run with:
//!
//! ```shell
//! cargo run --example messenger
//! ```

use nl3::{Nl3, ParseError};

fn messenger() -> Nl3 {
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
            // follow
            ("follow", "follow"),
            ("stalk", "follow"),
            ("watch", "follow"),
            // create
            ("creat", "create"),
            ("made", "create"),
            ("wrote", "create"),
            // send
            ("send", "send"),
            ("sent", "send"),
            ("mail", "send"),
            // receive
            ("retriev", "receive"),
            ("receiv", "receive"),
            ("reciev", "receive"),
            ("got", "receive"),
            // message
            ("messag", "message"),
            ("msg", "message"),
            ("contact", "message"),
        ])
        .build()
}

fn main() {
    let nl3 = messenger();

    println!("== Valid phrasings ==");
    for phrase in [
        "users who follow user 42",
        "users stalking user 42",
        "user bob created message 7",
        "user bob got message 7",
        "user bob contacted user jill",
    ] {
        print_triple(&nl3, phrase);
    }

    println!("\n== Errors ==");
    for phrase in ["", "dog jim hates cat sue"] {
        match nl3.parse(phrase) {
            Err(ParseError::EmptyInput) => {
                println!("{phrase:?} => EmptyInput");
            }
            Err(ParseError::InvalidTriple(t)) => {
                println!(
                    "{phrase:?} => InvalidTriple (subject type {:?})",
                    t.subject.ty
                );
            }
            Err(ParseError::AmbiguousType {
                predicate,
                candidates,
            }) => {
                println!("{phrase:?} => AmbiguousType for {predicate:?}: {candidates:?}");
            }
            Ok(t) => println!("{phrase:?} => unexpectedly ok: {t:?}"),
            // ParseError is #[non_exhaustive]; cover any future variants.
            Err(e) => println!("{phrase:?} => {e}"),
        }
    }
}

fn print_triple(nl3: &Nl3, phrase: &str) {
    match nl3.parse(phrase) {
        Ok(t) => println!(
            "{phrase:<30} => {}({}) -{}-> {}({})",
            t.subject.ty.as_deref().unwrap_or("?"),
            t.subject.value.as_deref().unwrap_or("-"),
            t.predicate.value.as_deref().unwrap_or("?"),
            t.object.ty.as_deref().unwrap_or("?"),
            t.object.value.as_deref().unwrap_or("-"),
        ),
        Err(e) => println!("{phrase:<30} => error: {e}"),
    }
}
