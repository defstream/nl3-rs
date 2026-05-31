//! Text classification.

use crate::tagger::Tagger;

/// The classification of an input phrase: its `(token, tag)` parts and trimmed text.
pub(crate) struct Classification {
    pub parts: Vec<(String, String)>,
    #[allow(dead_code)] // kept for parity with the original; not read by the pipeline
    pub text: String,
}

pub(crate) fn classify(text: &str, tagger: &dyn Tagger) -> Classification {
    Classification {
        parts: tagger.tag(text),
        text: text.trim().to_string(),
    }
}
