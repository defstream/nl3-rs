//! nl3's default [`LexiconTagger`] is a small closed-class lexicon. If you need
//! to recognize extra prepositions or WH-words — or want to delegate to a real
//! POS model — you can supply your own [`Tagger`].
//!
//! This example adds the archaic preposition "betwixt" to the set the tagger
//! treats as a preposition, so it is skipped during parsing.
//!
//! Run with:
//!
//! ```shell
//! cargo run --example custom_tagger
//! ```

use nl3::Nl3;
use nl3::tagger::Tagger;

/// A tagger that recognizes one extra preposition, then defers to the same
/// rules nl3's default uses.
struct ExtendedTagger;

impl Tagger for ExtendedTagger {
    fn tag(&self, text: &str) -> Vec<(String, String)> {
        text.split_whitespace()
            .map(|token| {
                let tag = match token.to_lowercase().as_str() {
                    "betwixt" => "IN", // our extra preposition
                    "by" | "from" | "on" | "to" | "with" | "of" => "IN",
                    "who" | "which" | "that" => "WDT",
                    t if !t.is_empty() && t.bytes().all(|b| b.is_ascii_digit()) => "CD",
                    _ => "NN",
                };
                (token.to_string(), tag.to_string())
            })
            .collect()
    }
}

fn main() {
    let nl3 = Nl3::builder()
        .grammar(["users follow users"])
        .vocabulary([("follow", "follow")])
        .tagger(ExtendedTagger)
        .build();

    // "betwixt" is treated as a preposition and skipped, so this still parses.
    let phrase = "users follow betwixt user 42";
    match nl3.parse(phrase) {
        Ok(t) => println!(
            "{phrase:?} => {}({}) -{}-> {}({})",
            t.subject.ty.as_deref().unwrap_or("?"),
            t.subject.value.as_deref().unwrap_or("-"),
            t.predicate.value.as_deref().unwrap_or("?"),
            t.object.ty.as_deref().unwrap_or("?"),
            t.object.value.as_deref().unwrap_or("-"),
        ),
        Err(e) => println!("{phrase:?} => error: {e}"),
    }
}
