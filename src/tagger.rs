//! Part-of-speech tagging. Ports the tagging half of `lib/text/classify.js`.
//!
//! The original used the npm `pos` package (a Brill tagger). Analysis of the
//! pipeline shows POS tags only ever change behavior in two ways: `reduce_parts`
//! skips prepositions (`IN`) and WH-words (any tag starting with `W`), and
//! numbers stay as values (`CD`). Entity *types* come from the grammar lexicon
//! and predicates from vocabulary stemming — not from tags. So a small
//! closed-class lexicon tagger reproduces every documented example.
//!
//! [`Tagger`] keeps this pluggable: a caller wanting general English POS
//! tagging can implement the trait over, e.g., `rust-bert`'s `POSModel`.

/// Tags text into `(token, Penn-Treebank-style tag)` pairs.
pub trait Tagger {
    /// Tag `text`, returning one `(token, tag)` pair per whitespace-separated
    /// token in order.
    fn tag(&self, text: &str) -> Vec<(String, String)>;
}

/// The default tagger: a closed-class lexicon. Recognizes prepositions (`IN`),
/// WH-words (`WDT`), and cardinal numbers (`CD`); everything else is `NN` and
/// falls through to the grammar/vocabulary logic.
#[derive(Debug, Clone, Copy, Default)]
pub struct LexiconTagger;

impl Tagger for LexiconTagger {
    fn tag(&self, text: &str) -> Vec<(String, String)> {
        text.split_whitespace()
            .map(|token| (token.to_string(), tag_word(token).to_string()))
            .collect()
    }
}

fn tag_word(word: &str) -> &'static str {
    if !word.is_empty() && word.bytes().all(|b| b.is_ascii_digit()) {
        return "CD";
    }
    match word.to_lowercase().as_str() {
        "by" | "from" | "on" | "for" | "to" | "with" | "of" | "in" | "at" | "into" | "onto"
        | "about" | "as" | "per" | "via" => "IN",
        "who" | "which" | "that" | "what" | "whom" | "whose" | "where" | "when" | "why" | "how" => {
            "WDT"
        }
        _ => "NN",
    }
}
