# nl3

[![CI](https://github.com/defstream/nl3/actions/workflows/ci.yml/badge.svg)](https://github.com/defstream/nl3/actions/workflows/ci.yml)
![MSRV](https://img.shields.io/badge/MSRV-1.95-blue)
![License](https://img.shields.io/badge/license-MIT-green)

**Natural language triples** — parse short plain-English *Subject–Predicate–Object*
phrases into validated triples.

You describe your domain with a **grammar** (the valid S-P-O relations) and a
**vocabulary** (word stems that map to predicates). nl3 then turns phrases like
`"user jack contacts user jill"` into a structured [`Triple`], validating each
one against the grammar and flipping reversed phrasings back into shape.

## What is a triple?

A triple represents a relationship as **Subject → Predicate → Object** — the data
model behind [RDF](https://en.wikipedia.org/wiki/Resource_Description_Framework)
and [triplestores](https://en.wikipedia.org/wiki/Triplestore). For example,
`"user jack contacts user jill"` becomes:

```text
subject:   { ty: "user", value: "jack" }
predicate: { value: "message" }
object:    { ty: "user", value: "jill" }
```

## Installation

```toml
[dependencies]
nl3-rs = "0.1"
```

The crate is imported as `nl3`:

```rust
use nl3::Nl3;
```

### Cargo features

- **`serde`** *(off by default)* — derive `Serialize`/`Deserialize` on `Triple`,
  `Term`, and `Predicate`:

  ```toml
  [dependencies]
  nl3-rs = { version = "0.1", features = ["serde"] }
  ```

### MSRV

The minimum supported Rust version is **1.95** (the crate uses edition 2024 and
let-chains). The MSRV is pinned via `rust-version` and verified in CI.

## Quick start

```rust
use nl3::Nl3;

let nl3 = Nl3::builder()
    .grammar(["users message users"])
    .vocabulary([
        ("msg", "message"),     // user jack msgs user jill
        ("messag", "message"),  // user jack messaged user jill
        ("contact", "message"), // user jack contacted user jill
    ])
    .build();

let triple = nl3.parse("user jack contacts user jill").unwrap();

assert_eq!(triple.subject.ty.as_deref(), Some("user"));
assert_eq!(triple.subject.value.as_deref(), Some("jack"));
assert_eq!(triple.predicate.value.as_deref(), Some("message"));
assert_eq!(triple.object.ty.as_deref(), Some("user"));
assert_eq!(triple.object.value.as_deref(), Some("jill"));
```

All of these phrasings produce the **same** triple:

```text
user jack msg user jill
user jack msgs user jill
user jack messaged user jill
user jack contacted user jill
user jack contacts user jill
```

## Concepts

### Grammar

Each grammar entry is a `"Subject Predicate Object"` string. All three words are
**singularized** when the rules are built, so `"users message users"` and
`"user message user"` are equivalent. The grammar defines which triples are
*valid*: a parsed phrase that does not fit any rule (in either direction) is
rejected.

### Vocabulary

The vocabulary maps **word stems** to predicates in your grammar. nl3 stems each
candidate word (classic Porter stemmer, after singularizing) and looks it up
here. This is how synonyms and tenses collapse onto one predicate:

```rust
.vocabulary([
    ("send", "send"),   // sends
    ("sent", "send"),   // sent
    ("mail", "send"),   // mailed
])
```

> **Tip:** vocabulary keys are *stems*, not whole words. `"messaged"` stems to
> `messag`, so the key is `"messag"`, not `"message"`.

### Flipping

If a phrase is grammatically valid only when subject and object are swapped, nl3
flips it for you, so reversed phrasings still yield the correctly-oriented triple.

### Type inference

If a phrase omits an entity type, nl3 infers it from the grammar via the matched
predicate. Given `"users message users"`, the predicate `message` only ever
relates a `user` to a `user`, so a bare phrase resolves both ends:

```rust
let nl3 = Nl3::builder()
    .grammar(["users message users"])
    .vocabulary([("contact", "message")])
    .build();

let t = nl3.parse("jack contacts jill").unwrap();
assert_eq!(t.subject.ty.as_deref(), Some("user")); // inferred
assert_eq!(t.subject.value.as_deref(), Some("jack"));
assert_eq!(t.object.ty.as_deref(), Some("user")); // inferred
assert_eq!(t.object.value.as_deref(), Some("jill"));
```

A type is inferred only when the predicate determines it. When a predicate maps
to **more than one** candidate type (e.g. both `"users message users"` and
`"admins message users"`), the [`Ambiguity`] policy decides:

```rust
use nl3::{Ambiguity, Nl3};

let nl3 = Nl3::builder()
    .grammar(["users message users", "admins message users"])
    .vocabulary([("contact", "message")])
    .ambiguity(Ambiguity::FirstMatch) // default: take the first matching rule
    .build();

// "users message users" is declared first, so jack is a user.
assert_eq!(
    nl3.parse("jack contacts jill").unwrap().subject.ty.as_deref(),
    Some("user"),
);
```

- `Ambiguity::FirstMatch` *(default)* — use the first matching rule, in grammar
  declaration order.
- `Ambiguity::Error` — return [`ParseError::AmbiguousType`] (with the predicate
  and candidate types) instead of guessing.

Spelling the type out (`"user jack contacts user jill"`) always bypasses
inference.

## The `Triple` type

```rust
pub struct Triple {
    pub subject: Term,
    pub predicate: Predicate,
    pub object: Term,
}

pub struct Term {
    pub ty: Option<String>,    // entity type, e.g. "user"
    pub value: Option<String>, // entity value, e.g. "jack"
}

pub struct Predicate {
    pub ty: Option<String>,    // always None (not implemented upstream)
    pub value: Option<String>, // the mapped predicate, e.g. "message"
}
```

Fields are `Option` because a phrase may omit a value (e.g. `"users who follow
user 42"` has no subject value). The field is named `ty` because `type` is a
reserved word in Rust.

## Errors

[`Nl3::parse`] returns `Result<Triple, ParseError>`:

```rust
pub enum ParseError {
    EmptyInput,                 // text was empty or whitespace
    InvalidTriple(Box<Triple>), // didn't match the grammar in either direction
    AmbiguousType {             // an omitted type couldn't be inferred uniquely
        predicate: String,
        candidates: Vec<String>,
    },
}
```

```rust
use nl3::{Nl3, ParseError};

let nl3 = Nl3::builder()
    .grammar(["users follow users"])
    .vocabulary([("follow", "follow")])
    .build();

assert!(matches!(nl3.parse(""), Err(ParseError::EmptyInput)));
assert!(matches!(
    nl3.parse("dog jim hates cat sue"),
    Err(ParseError::InvalidTriple(_)),
));
```

## Custom taggers

Under the hood nl3 applies a part-of-speech tagger to skip prepositions
(`by`, `from`, …) and WH-words (`who`, `which`, `that`). The default
[`LexiconTagger`] is a small closed-class lexicon that covers the common cases
with zero heavy dependencies.

To recognize more words — or to delegate to a real POS model such as
[`rust-bert`](https://crates.io/crates/rust-bert) — implement the [`Tagger`]
trait and pass it to the builder:

```rust
use nl3::{tagger::Tagger, Nl3};

struct MyTagger;

impl Tagger for MyTagger {
    fn tag(&self, text: &str) -> Vec<(String, String)> {
        text.split_whitespace()
            .map(|t| (t.to_string(), "NN".to_string())) // ... your logic
            .collect()
    }
}

let nl3 = Nl3::builder()
    .grammar(["users follow users"])
    .vocabulary([("follow", "follow")])
    .tagger(MyTagger)
    .build();
```

## Examples

Runnable examples live in [`examples/`](examples/):

| Example | What it shows |
|---|---|
| [`basic`](examples/basic.rs) | One grammar rule; many phrasings → one triple |
| [`messenger`](examples/messenger.rs) | A fuller domain plus both error modes |
| [`ambiguity`](examples/ambiguity.rs) | Disallowing ambiguous inference with [`Ambiguity::Error`] |
| [`custom_tagger`](examples/custom_tagger.rs) | Supplying your own [`Tagger`] |

```shell
cargo run --example basic
cargo run --example messenger
cargo run --example ambiguity
cargo run --example custom_tagger
```

## Development

A [`Makefile`](Makefile) wraps the common Cargo tasks:

```shell
make          # fmt + lint + test
make test     # unit, integration, and doctests
make test-all # tests with all features (incl. serde)
make lint     # clippy across all features, warnings denied
make bench    # criterion benchmarks
make doc      # build the API docs
make ci       # fmt-check + lint + test + test-all
```

CI (GitHub Actions, [`.github/workflows/ci.yml`](.github/workflows/ci.yml)) runs
the same checks on stable and the pinned MSRV, across default and all features.

## Performance

`parse()` is allocation-light and runs in single-digit microseconds. The
predicate stemmer — the heaviest per-token step — is memoized in a per-thread
cache, so repeated words (the common case) cost a hash lookup rather than a full
Porter-stemmer pass. Measured with the [`parse`](benches/parse.rs) benchmark
(`cargo bench`), this roughly halves parse time on cache hits. The cache keeps
`parse(&self)` lock-free and is bounded to avoid unbounded growth.

## License

MIT — see [LICENSE](LICENSE).