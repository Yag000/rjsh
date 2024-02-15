use rjsh::editor::RjshEditor;
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
        let prompt = get_prompt(&shell).unwrap_or_else(|_| String::from("$ "));
        let readline = rl.readline(prompt.as_str());
        match readline {
            Ok(line) => {
                if line.trim() == "" {
                    continue;
                }
                let mut parser = Parser::new(line.clone());
                if let Some(command) = parser.parse_command() {
                    if !command.name.is_empty() {
                        rl.add_history_entry(line)?;
                        shell.execute_command(&command)?;
                    }
                } else {
                    eprintln!("syntax error");
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    rl.save_history(&history_path)?;

    std::process::exit(shell.last_exit_code());
}
