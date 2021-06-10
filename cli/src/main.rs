use cachem::v2::{Command, ConnectionPool};
use cachem_example::*;
use rustyline::error::ReadlineError;
use rustyline::hint::{Hint, Hinter};
use rustyline::highlight::Highlighter;
use rustyline::{Config, Context, Editor};
use rustyline_derive::{Completer, Helper, Validator};
use std::collections::HashSet;
use std::borrow::Cow::{self, Borrowed, Owned};

enum CacheName {
    Item,
}
impl Into<u8> for CacheName {
    fn into(self) -> u8 {
        0u8
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = ConnectionPool::new("0.0.0.0:55555", 1).await?;

    let config = Config::builder()
        .color_mode(rustyline::ColorMode::Forced)
        .edit_mode(rustyline::EditMode::Vi)
        .build();

    let h = DIYHinter { hints: diy_hints(), prompt: "Cachem > ".into() };

    let mut rl: Editor<DIYHinter> = Editor::with_config(config);
    rl.set_helper(Some(h));

    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline("Cachem > ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let mut con = pool.acquire().await.unwrap();
                let res = con.keys::<_, u32>(CacheName::Item).await?;
                println!("Line: {:?}", res);
            },
            Err(ReadlineError::Interrupted) => {
                break
            },
            Err(ReadlineError::Eof) => {
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
    rl.save_history("history.txt").unwrap();

    Ok(())
}

#[derive(Completer, Helper, Validator)]
struct DIYHinter {
    hints:  HashSet<CommandHint>,
    prompt: String,
}

#[derive(Hash, Debug, PartialEq, Eq)]
struct CommandHint {
    display: String,
    complete_up_to: usize,
}

impl Hint for CommandHint {
    fn display(&self) -> &str {
        &self.display
    }

    fn completion(&self) -> Option<&str> {
        if self.complete_up_to > 0 {
            Some(&self.display[..self.complete_up_to])
        } else {
            None
        }
    }
}

impl CommandHint {
    fn new(text: &str, complete_up_to: &str) -> CommandHint {
        assert!(text.starts_with(complete_up_to));
        CommandHint {
            display: text.into(),
            complete_up_to: complete_up_to.len(),
        }
    }

    fn suffix(&self, strip_chars: usize) -> CommandHint {
        CommandHint {
            display: self.display[strip_chars..].to_owned(),
            complete_up_to: self.complete_up_to.saturating_sub(strip_chars),
        }
    }
}

impl Highlighter for DIYHinter {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _: bool,
    ) -> Cow<'b, str> {
        Borrowed(&self.prompt)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1B[0;34m".to_owned() + hint + "\x1B[m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        Borrowed(&line)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        false
    }
}

impl Hinter for DIYHinter {
    type Hint = CommandHint;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<CommandHint> {
        if pos < line.len() {
            return None;
        }

        self.hints
            .iter()
            .filter_map(|hint| {
                // expect hint after word complete, like redis cli, add condition:
                // line.ends_with(" ")
                if pos > 0 && hint.display.starts_with(&line[..pos]) {
                    Some(hint.suffix(pos))
                } else {
                    None
                }
            })
            .next()
    }
}

fn diy_hints() -> HashSet<CommandHint> {
    let mut set = HashSet::new();
    set.insert(CommandHint::new("PING", "PING"));
    set.insert(CommandHint::new("GET idx", "GET "));
    set
}

