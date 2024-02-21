use rjsh::editor::RjshEditor;
use rjsh::exec::execute_command;
use rjsh::parser::Parser;
use rjsh::prompt::get_prompt;
use rjsh::shell::Shell;
use rustyline::error::ReadlineError;

fn main() -> anyhow::Result<()> {
    let mut rl = RjshEditor::new()?;

    let home_dir = std::env::var("HOME")?;
    let history_path = format!("{}/.rjsh_history", home_dir);
    if rl.load_history(&history_path).is_err() {
        std::fs::File::create(&history_path)?;
    }
    let mut shell = rjsh::shell::DefaultShell::default();

    while !shell.should_exit() {
        shell.update_jobs();
        let prompt = get_prompt(&shell).unwrap_or_else(|_| String::from("$ "));
        let readline = rl.readline(prompt.as_str());
        match readline {
            Ok(line) => {
                if line.trim() == "" {
                    continue;
                }
                let mut parser = Parser::new(line.clone());
                match parser.parse_command() {
                    Ok(command) => {
                        if !command.name.is_empty() {
                            // TODO: Do not add a successful exit to the history
                            rl.add_history_entry(line)?;

                            match execute_command(&mut shell, command) {
                                Ok(_) => {}
                                Err(e) => eprintln!("rjsh: {e}"),
                            }
                        }
                    }
                    Err(e) => eprintln!("{e}"),
                }
            }
            Err(ReadlineError::Interrupted) => {
                break;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("rjsh: {:?}", err);
                break;
            }
        }
    }

    rl.save_history(&history_path)?;

    std::process::exit(shell.last_exit_code());
}
