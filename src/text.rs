//! Text normalization helpers.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;

/// Singularize text, e.g. `cats` -> `cat`. Ports `lib/text/singularize.js`.
///
/// Mirrors the JS `(text && singular(text)) || text`: empty input is returned
/// unchanged, and a singularizer that yields an empty string falls back to the
/// original.
pub fn singularize(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let singular = pluralizer::pluralize(text, 1, false);
    if singular.is_empty() {
        text.to_string()
    } else {
        singular
    }
}

/// Upper bound on the per-thread stem cache, to keep memory bounded if a caller
/// throws a huge vocabulary of distinct words at `parse`. When reached, the
/// cache simply stops growing (existing entries still serve hits).
const STEM_CACHE_CAP: usize = 4096;

thread_local! {
    /// Memoizes `stem(singularize(word))` per thread. The vocabulary lookup key
    /// for a given input word never changes, and real workloads repeat words
    /// (e.g. `"user"`, `"contacts"`) across calls, so this turns the heaviest
    /// per-token step into a hash lookup. Thread-local keeps `parse(&self)`
    /// lock-free.
    static STEM_CACHE: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

/// The vocabulary key for `word`: `stem(singularize(word))`, memoized per thread.
pub(crate) fn stem_key(word: &str) -> String {
    if let Some(hit) = STEM_CACHE.with(|c| c.borrow().get(word).cloned()) {
        return hit;
    }
    let key = stem(&singularize(word));
    STEM_CACHE.with(|c| {
        let mut cache = c.borrow_mut();
        if cache.len() < STEM_CACHE_CAP {
            cache.insert(word.to_string(), key.clone());
        }
    });
    key
}

/// Returns true if `text` appears to be plural. Ports `lib/text/is/plural.js`.
pub fn is_plural(text: &str) -> bool {
    let ends_with_s = text
        .chars()
        .last()
        .map(|c| c.eq_ignore_ascii_case(&'s'))
        .unwrap_or(false);
    let differs_from_singular = text.to_lowercase() != singularize(text).to_lowercase();
    differs_from_singular || ends_with_s
}

// --- Porter stemmer (port of npm `stemmer` 0.1.4) -------------------------

// Consonant/vowel measure patterns.
static GT0: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([^aeiou][^aeiouy]*)?([aeiouy][aeiou]*)([^aeiou][^aeiouy]*)").unwrap()
});
static EQ1: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([^aeiou][^aeiouy]*)?([aeiouy][aeiou]*)([^aeiou][^aeiouy]*)([aeiouy][aeiou]*)?$")
        .unwrap()
});
static GT1: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([^aeiou][^aeiouy]*)?(([aeiouy][aeiou]*)([^aeiou][^aeiouy]*)){2,}").unwrap()
});
static VOWEL_IN_STEM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([^aeiou][^aeiouy]*)?[aeiouy]").unwrap());
static CONSONANT_LIKE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([^aeiou][^aeiouy]*)[aeiouy][^aeiouwxy]$").unwrap());

// Suffix patterns.
static SFX_LL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"ll$").unwrap());
static SFX_E: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(.+?)e$").unwrap());
static SFX_Y: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(.+?)y$").unwrap());
static SFX_ION: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(.+?(s|t))(ion)$").unwrap());
static SFX_ED_OR_ING: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(.+?)(ed|ing)$").unwrap());
static SFX_AT_BL_IZ: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(at|bl|iz)$").unwrap());
static SFX_EED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(.+?)eed$").unwrap());
static SFX_S: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^.+?[^s]s$").unwrap());
static SFX_SSES_OR_IES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^.+?(ss|i)es$").unwrap());
static STEP2: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(.+?)(ational|tional|enci|anci|izer|bli|alli|entli|eli|ousli|ization|ation|ator|alism|iveness|fulness|ousness|aliti|iviti|biliti|logi)$").unwrap()
});
static STEP3: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(.+?)(icate|ative|alize|iciti|ical|ful|ness)$").unwrap());
static STEP4: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(.+?)(al|ance|ence|er|ic|able|ible|ant|ement|ment|ent|ou|ism|ate|iti|ous|ive|ize)$",
    )
    .unwrap()
});

fn step2list(suffix: &str) -> &'static str {
    match suffix {
        "ational" => "ate",
        "tional" => "tion",
        "enci" => "ence",
        "anci" => "ance",
        "izer" => "ize",
        "bli" => "ble",
        "alli" => "al",
        "entli" => "ent",
        "eli" => "e",
        "ousli" => "ous",
        "ization" => "ize",
        "ation" => "ate",
        "ator" => "ate",
        "alism" => "al",
        "iveness" => "ive",
        "fulness" => "ful",
        "ousness" => "ous",
        "aliti" => "al",
        "iviti" => "ive",
        "biliti" => "ble",
        "logi" => "log",
        _ => "",
    }
}

fn step3list(suffix: &str) -> &'static str {
    match suffix {
        "icate" => "ic",
        "ative" => "",
        "alize" => "al",
        "iciti" => "ic",
        "ical" => "ic",
        "ful" => "",
        "ness" => "",
        _ => "",
    }
}

/// The JS `/([^aeiouylsz])\1$/` test — a backreference the `regex` crate can't
/// express, implemented directly: the last two characters are equal and not in
/// the excluded class.
fn multi_consonant_like(s: &str) -> bool {
    let b = s.as_bytes();
    let n = b.len();
    if n < 2 {
        return false;
    }
    let last = b[n - 1];
    last == b[n - 2]
        && !matches!(
            last,
            b'a' | b'e' | b'i' | b'o' | b'u' | b'y' | b'l' | b's' | b'z'
        )
}

/// Porter-stem `value`, matching npm `stemmer` 0.1.4 exactly.
pub(crate) fn stem(value: &str) -> String {
    let mut value = value.to_lowercase();

    // Exit early on very short words.
    if value.chars().count() < 3 {
        return value;
    }

    // Detect an initial `y` and mask it so it never counts as a vowel.
    let first_char_was_y = value.as_bytes()[0] == b'y';
    if first_char_was_y {
        value = format!("Y{}", &value[1..]);
    }

    // Step 1a.
    if SFX_SSES_OR_IES.is_match(&value) {
        value.truncate(value.len() - 2);
    } else if SFX_S.is_match(&value) {
        value.truncate(value.len() - 1);
    }

    // Step 1b.
    if SFX_EED.is_match(&value) {
        let stem = SFX_EED.replace(&value, "$1").into_owned();
        if GT0.is_match(&stem) {
            value.truncate(value.len() - 1);
        }
    } else if SFX_ED_OR_ING.is_match(&value) {
        let stem = SFX_ED_OR_ING.replace(&value, "$1").into_owned();
        if VOWEL_IN_STEM.is_match(&stem) {
            value = stem;
            if SFX_AT_BL_IZ.is_match(&value) {
                value.push('e');
            } else if multi_consonant_like(&value) {
                value.truncate(value.len() - 1);
            } else if CONSONANT_LIKE.is_match(&value) {
                value.push('e');
            }
        }
    }

    // Step 1c.
    if SFX_Y.is_match(&value) {
        let stem = SFX_Y.replace(&value, "$1").into_owned();
        if VOWEL_IN_STEM.is_match(&stem) {
            value = format!("{stem}i");
        }
    }

    // Step 2.
    let next = STEP2.captures(&value).and_then(|caps| {
        let stem = caps[1].to_string();
        if GT0.is_match(&stem) {
            Some(format!("{stem}{}", step2list(&caps[2])))
        } else {
            None
        }
    });
    if let Some(n) = next {
        value = n;
    }

    // Step 3.
    let next = STEP3.captures(&value).and_then(|caps| {
        let stem = caps[1].to_string();
        if GT0.is_match(&stem) {
            Some(format!("{stem}{}", step3list(&caps[2])))
        } else {
            None
        }
    });
    if let Some(n) = next {
        value = n;
    }

    // Step 4.
    let next = STEP4.captures(&value).and_then(|caps| {
        let stem = caps[1].to_string();
        if GT1.is_match(&stem) {
            Some(stem)
        } else {
            None
        }
    });
    if let Some(n) = next {
        value = n;
    }
    let next = SFX_ION.captures(&value).and_then(|caps| {
        let stem = caps[1].to_string();
        if GT1.is_match(&stem) {
            Some(stem)
        } else {
            None
        }
    });
    if let Some(n) = next {
        value = n;
    }

    // Step 5.
    let next = SFX_E.captures(&value).and_then(|caps| {
        let stem = caps[1].to_string();
        if GT1.is_match(&stem) || (EQ1.is_match(&stem) && !CONSONANT_LIKE.is_match(&stem)) {
            Some(stem)
        } else {
            None
        }
    });
    if let Some(n) = next {
        value = n;
    }

    if SFX_LL.is_match(&value) && GT1.is_match(&value) {
        value.truncate(value.len() - 1);
    }

    // Restore the masked initial `y`.
    if first_char_was_y {
        value = format!("y{}", &value[1..]);
    }

    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stems_match_original_toolchain() {
        // Ground truth captured from npm `stemmer` 0.1.4 + `pluralize` 1.2.1
        // (key = stem(singularize(word))), the exact values nl3's vocabulary
        // keys are built from.
        let cases = [
            ("follow", "follow"),
            ("follows", "follow"),
            ("following", "follow"),
            ("followed", "follow"),
            ("stalk", "stalk"),
            ("stalking", "stalk"),
            ("watch", "watch"),
            ("watches", "watch"),
            ("watching", "watch"),
            ("creates", "creat"),
            ("created", "creat"),
            ("create", "creat"),
            ("sends", "send"),
            ("sent", "sent"),
            ("sended", "send"),
            ("mailed", "mail"),
            ("got", "got"),
            ("received", "receiv"),
            ("retrieved", "retriev"),
            ("recieved", "reciev"),
            ("messaged", "messag"),
            ("msgs", "msg"),
            ("contacted", "contact"),
            ("contacts", "contact"),
        ];
        for (word, expected) in cases {
            assert_eq!(stem(&singularize(word)), expected, "key for {word:?}");
        }
    }

    #[test]
    fn singularizes_grammar_nouns() {
        assert_eq!(singularize("users"), "user");
        assert_eq!(singularize("messages"), "message");
        assert_eq!(singularize("content"), "content");
    }

    #[test]
    fn detects_plurals() {
        assert!(is_plural("cats"));
        assert!(is_plural("users"));
        assert!(!is_plural("cat"));
        assert!(!is_plural("user"));
    }
}
