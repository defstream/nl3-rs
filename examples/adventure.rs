//! A tiny text-adventure game driven by nl3.
//!
//! ## How nl3 is used here
//!
//! nl3 models relations as **subject-type → predicate → object-type**, where a
//! specific thing is a *value* of a type. This game mirrors that: the world is
//! made of categories (`item`, `foe`, `exit`) whose members are named things
//! (`key`/`sword`/`torch`, `troll`, `gate`). The grammar encodes the game's
//! *affordances by category*:
//!
//! ```text
//! player take   item   // you can take items
//! player attack foe    // you can attack foes
//! player open   exit   // you can open exits
//! ```
//!
//! A verb/category pair the grammar doesn't list — `take foe`, say — is rejected
//! by the parser itself, so "you can't take the troll" needs no hand-written
//! check.
//!
//! The trick that makes this ergonomic: nl3 can only set an object *type* it
//! sees in the text, but the player types `take key`, not `take item key`. So
//! the game looks up each noun's category and injects it, parsing
//! `player take item key`. From the triple it then reads the verb
//! (`predicate.value`), the category (`object.ty`), and the named thing
//! (`object.value`). Prepending `player` also keeps the predicate off index 0,
//! where nl3 faithfully reproduces the original library's quirk of not reading
//! an object.
//!
//! Bareword commands that aren't Subject-Predicate-Object (`north`, `look`,
//! `inventory`, `quit`) are matched directly — that split plays to nl3's
//! strengths rather than forcing every input through it.
//!
//! Run with:
//!
//! ```shell
//! cargo run --example adventure
//! ```

use std::collections::HashMap;
use std::io::{self, Write};

use nl3::Nl3;

/// Where a takeable item currently is.
#[derive(PartialEq)]
enum Where {
    Room(usize),
    Inventory,
}

struct Room {
    name: &'static str,
    desc: &'static str,
    /// direction -> destination room index
    exits: HashMap<&'static str, usize>,
}

const GATE: usize = 3;

/// The takeable items and where each one starts.
const ITEMS: &[(&str, usize)] = &[("key", 0), ("torch", 1), ("sword", 2)];

/// Every noun the player can name, paired with its grammar category. This table
/// is the bridge between free-typed nouns and nl3's category types: the game
/// looks a noun up here to inject its category before parsing.
const NOUNS: &[(&str, &str)] = &[
    ("key", "item"),
    ("sword", "item"),
    ("torch", "item"),
    ("troll", "foe"),
    ("gate", "exit"),
];

fn category_of(noun: &str) -> Option<&'static str> {
    NOUNS.iter().find(|(n, _)| *n == noun).map(|(_, c)| *c)
}

struct Game {
    rooms: Vec<Room>,
    here: usize,
    items: HashMap<&'static str, Where>,
    troll_alive: bool,
    won: bool,
}

impl Game {
    fn new() -> Self {
        let room = |name, desc, exits: &[(&'static str, usize)]| Room {
            name,
            desc,
            exits: exits.iter().copied().collect(),
        };

        let rooms = vec![
            room(
                "Cell",
                "A damp stone cell. A rusty door leads east.",
                &[("east", 1)],
            ),
            room(
                "Corridor",
                "A torchlit corridor. Passages run north and east, and back west to your cell.",
                &[("west", 0), ("north", 2), ("east", GATE)],
            ),
            room(
                "Armory",
                "An old armory. Weapon racks line the walls. The corridor is south.",
                &[("south", 1)],
            ),
            room(
                "Gate",
                "The great castle gate - your way out. The corridor lies west.",
                &[("west", 1)],
            ),
        ];

        let items = ITEMS
            .iter()
            .map(|(name, start)| (*name, Where::Room(*start)))
            .collect();

        Game {
            rooms,
            here: 0,
            items,
            troll_alive: true,
            won: false,
        }
    }

    fn has(&self, item: &str) -> bool {
        self.items.get(item) == Some(&Where::Inventory)
    }

    /// The static name of a known item, or `None` if `noun` isn't an item.
    fn item_name(noun: &str) -> Option<&'static str> {
        ITEMS.iter().map(|(n, _)| *n).find(|n| *n == noun)
    }

    fn describe(&self) {
        let room = &self.rooms[self.here];
        println!("\n== {} ==", room.name);
        println!("{}", room.desc);

        let here: Vec<&str> = self
            .items
            .iter()
            .filter(|(_, w)| **w == Where::Room(self.here))
            .map(|(name, _)| *name)
            .collect();
        if !here.is_empty() {
            println!("You see: {}.", here.join(", "));
        }
        if self.here == GATE && self.troll_alive {
            println!("A hulking TROLL blocks the gate, snarling.");
        }

        let mut exits: Vec<&str> = room.exits.keys().copied().collect();
        exits.sort_unstable();
        println!("Exits: {}.", exits.join(", "));
    }

    fn go(&mut self, dir: &str) {
        match self.rooms[self.here].exits.get(dir) {
            Some(&dest) => {
                self.here = dest;
                self.describe();
            }
            None => println!("You can't go {dir}."),
        }
    }

    fn inventory(&self) {
        let held: Vec<&str> = self
            .items
            .iter()
            .filter(|(_, w)| **w == Where::Inventory)
            .map(|(name, _)| *name)
            .collect();
        if held.is_empty() {
            println!("You are empty-handed.");
        } else {
            println!("You are carrying: {}.", held.join(", "));
        }
    }

    /// Apply a parsed command. `verb` and `noun` come straight from the triple's
    /// predicate value and object value.
    fn act(&mut self, verb: &str, noun: &str) {
        match verb {
            "take" => match Game::item_name(noun) {
                Some(item) => match self.items.get(item) {
                    Some(Where::Room(r)) if *r == self.here => {
                        self.items.insert(item, Where::Inventory);
                        println!("You take the {item}.");
                    }
                    Some(Where::Inventory) => println!("You already have the {item}."),
                    _ => println!("There is no {item} here."),
                },
                None => println!("You can't carry that."),
            },
            "drop" => match Game::item_name(noun) {
                Some(item) if self.has(item) => {
                    self.items.insert(item, Where::Room(self.here));
                    println!("You drop the {item}.");
                }
                Some(item) => println!("You aren't carrying a {item}."),
                None => println!("You aren't carrying that."),
            },
            "attack" => {
                // Grammar guarantees `noun` is a foe; `troll` is the only one.
                if self.here != GATE || !self.troll_alive {
                    println!("There is nothing here to attack.");
                } else if self.has("sword") {
                    self.troll_alive = false;
                    println!("You swing the sword. The troll roars and flees into the dark!");
                } else {
                    println!("The troll shrugs off your bare fists. You need a weapon.");
                }
            }
            "open" => {
                // Grammar guarantees `noun` is an exit; `gate` is the only one.
                if self.here != GATE {
                    println!("There is no {noun} here.");
                } else if self.troll_alive {
                    println!("The troll blocks the gate. You'll have to deal with it first.");
                } else if self.has("key") {
                    println!("You turn the key. The gate groans open and daylight pours in...");
                    self.won = true;
                } else {
                    println!("The gate is locked. You need a key.");
                }
            }
            _ => println!("You can't do that."),
        }
    }
}

/// Build the nl3 parser. The grammar is written in terms of *categories*
/// (`item`, `foe`, `exit`), so each verb maps to exactly one category.
fn build_parser() -> Nl3 {
    Nl3::builder()
        .grammar([
            "player take item",
            "player drop item",
            "player attack foe",
            "player open exit",
        ])
        .vocabulary([
            // verb synonyms -> canonical predicate (keys are word stems)
            ("take", "take"),
            ("grab", "take"),
            ("get", "take"),
            ("pick", "take"),
            ("pickup", "take"),
            ("drop", "drop"),
            ("attack", "attack"),
            ("hit", "attack"),
            ("kill", "attack"),
            ("fight", "attack"),
            ("open", "open"),
        ])
        .build()
}

fn normalize_direction(word: &str) -> Option<&'static str> {
    match word {
        "n" | "north" => Some("north"),
        "s" | "south" => Some("south"),
        "e" | "east" => Some("east"),
        "w" | "west" => Some("west"),
        _ => None,
    }
}

fn help() {
    println!("Commands:");
    println!("  go <dir> / n, s, e, w   move around");
    println!("  take <item>, drop <item>");
    println!("  attack <foe>, open <thing>");
    println!("  look (l), inventory (i), help, quit");
}

/// Parse a `verb noun` command via nl3, returning `(verb, noun)` on success.
///
/// The noun's category is looked up and injected so nl3 can assign the object
/// type: `take key` is parsed as `player take item key`. Returns `None` if the
/// noun is unknown or the verb/category pair isn't a valid affordance.
fn parse_command(nl3: &Nl3, verb_word: &str, noun_word: &str) -> Option<(String, String)> {
    let category = category_of(noun_word)?;
    let command = format!("player {verb_word} {category} {noun_word}");
    let triple = nl3.parse(&command).ok()?;
    let verb = triple.predicate.value?;
    let noun = triple.object.value?;
    Some((verb, noun))
}

fn main() {
    let nl3 = build_parser();
    let mut game = Game::new();

    println!("=== THE RUSTY GATE ===");
    println!("Escape the keep. Type 'help' for commands.");
    game.describe();

    let stdin = io::stdin();
    let mut line = String::new();

    loop {
        print!("\n> ");
        let _ = io::stdout().flush();

        line.clear();
        if stdin.read_line(&mut line).unwrap_or(0) == 0 {
            break; // EOF
        }
        let input = line.trim().to_lowercase();
        if input.is_empty() {
            continue;
        }

        // --- Bareword commands (not Subject-Predicate-Object). ---
        let mut words = input.split_whitespace();
        let first = words.next().unwrap();

        match first {
            "quit" | "exit" | "q" => {
                println!("You give up. Farewell.");
                break;
            }
            "help" | "?" => {
                help();
                continue;
            }
            "look" | "l" => {
                game.describe();
                continue;
            }
            "inventory" | "inv" | "i" => {
                game.inventory();
                continue;
            }
            "go" => {
                match words.next().and_then(normalize_direction) {
                    Some(dir) => game.go(dir),
                    None => println!("Go where?"),
                }
                continue;
            }
            _ => {}
        }

        // A bare direction like "north".
        if let Some(dir) = normalize_direction(first) {
            game.go(dir);
            continue;
        }

        // --- Structured "verb noun" commands, parsed by nl3. ---
        match words.next() {
            Some(noun) => match parse_command(&nl3, first, noun) {
                Some((verb, noun)) => game.act(&verb, &noun),
                None => println!("You can't do that."),
            },
            None => println!("{first} what?"),
        }

        if game.won {
            println!("\n*** You have escaped. YOU WIN! ***");
            break;
        }
    }
}
