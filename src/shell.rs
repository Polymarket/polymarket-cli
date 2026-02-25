use clap::Parser as _;

pub async fn run_shell() {
    println!();
    println!("  Polymarket CLI · Interactive Shell");
    println!("  Type 'help' for commands, 'exit' to quit.");
    println!();

    let mut rl = match rustyline::DefaultEditor::new() {
        Ok(rl) => rl,
        Err(e) => {
            eprintln!("Failed to initialize shell: {e}");
            return;
        }
    };

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
