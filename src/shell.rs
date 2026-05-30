use clap::{CommandFactory, Parser as _};
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Editor, Helper};

/// Tab-completion helper for the interactive shell.
///
/// Builds the completion tree from clap's `Command` definition so it stays
/// in sync with the CLI automatically — no manual list to maintain.
struct ShellCompleter {
    /// `(command_name, [subcommand_names])` pairs derived from `Cli::command()`.
    commands: Vec<(String, Vec<String>)>,
}

impl ShellCompleter {
    fn new() -> Self {
        let cmd = crate::Cli::command();
        let commands = cmd
            .get_subcommands()
            .filter(|sub| {
                let name = sub.get_name();
                // "shell" and "setup" are blocked inside the REPL, skip them.
                name != "shell" && name != "setup"
            })
            .map(|sub| {
                let name = sub.get_name().to_string();
                let children: Vec<String> = sub
                    .get_subcommands()
                    .map(|s| s.get_name().to_string())
                    .collect();
                (name, children)
            })
            .collect();

        Self { commands }
    }
}

impl Completer for ShellCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let input = &line[..pos];
        let words: Vec<&str> = input.split_whitespace().collect();
        let trailing_space = input.ends_with(' ');

        match (words.len(), trailing_space) {
            // Nothing typed yet, or still typing the first word.
            (0, _) | (1, false) => {
                let prefix = words.first().copied().unwrap_or("");
                let start = pos - prefix.len();

                let matches: Vec<Pair> = self
                    .commands
                    .iter()
                    .map(|(name, _)| name.as_str())
                    .chain(["help", "exit", "quit"])
                    .filter(|c| c.starts_with(prefix))
                    .map(|c| Pair {
                        display: c.to_string(),
                        replacement: c.to_string(),
                    })
                    .collect();

                Ok((start, matches))
            }

            // First word complete, typing (or about to type) the subcommand.
            (1, true) | (2, false) => {
                let cmd = words[0];
                let prefix = if words.len() == 2 && !trailing_space {
                    words[1]
                } else {
                    ""
                };
                let start = pos - prefix.len();

                if let Some((_, subs)) = self.commands.iter().find(|(name, _)| name == cmd) {
                    let matches: Vec<Pair> = subs
                        .iter()
                        .filter(|s| s.starts_with(prefix))
                        .map(|s| Pair {
                            display: s.to_string(),
                            replacement: s.to_string(),
                        })
                        .collect();
                    Ok((start, matches))
                } else {
                    Ok((pos, vec![]))
                }
            }

            // Beyond the subcommand — no further completion for now.
            _ => Ok((pos, vec![])),
        }
    }
}

impl Hinter for ShellCompleter {
    type Hint = String;

    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
        None
    }
}

impl Highlighter for ShellCompleter {}
impl Validator for ShellCompleter {}
impl Helper for ShellCompleter {}

pub async fn run_shell() -> anyhow::Result<()> {
    println!();
    println!("  Polymarket CLI · Interactive Shell");
    println!("  Type 'help' for commands, 'exit' to quit.");
    println!("  Tab completion is available for commands.");
    println!();

    let helper = ShellCompleter::new();
    let mut rl = Editor::new()?;
    rl.set_helper(Some(helper));

    loop {
        match rl.readline("polymarket> ") {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if line == "exit" || line == "quit" {
                    break;
                }

                let _ = rl.add_history_entry(line);

                let args = split_args(line);
                let mut full_args = vec!["polymarket".to_string()];
                full_args.extend(args);

                if let Some(cmd) = full_args.get(1) {
                    if cmd == "shell" {
                        println!("Already in shell mode.");
                        continue;
                    }
                    if cmd == "setup" {
                        println!("Run 'polymarket setup' outside the shell.");
                        continue;
                    }
                }

                match crate::Cli::try_parse_from(&full_args) {
                    Ok(cli) => {
                        let output = cli.output;
                        if let Err(e) = crate::run(cli).await {
                            crate::output::print_error(&e, output);
                        }
                    }
                    Err(e) => {
                        let _ = e.print();
                    }
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => continue,
            Err(rustyline::error::ReadlineError::Eof) => break,
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            }
        }
    }

    println!("Goodbye!");
    Ok(())
}

fn split_args(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for c in input.chars() {
        match c {
            '"' => in_quotes = !in_quotes,
            ' ' if !in_quotes => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(c),
        }
    }
    if !current.is_empty() {
        args.push(current);
    }
    args
}
