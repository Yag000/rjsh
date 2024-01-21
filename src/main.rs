use rjsh::parser::Parser;
use rjsh::shell::Shell;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

fn main() -> anyhow::Result<()> {
    let mut rl = DefaultEditor::new()?;

    let home_dir = std::env::var("HOME")?;
    let history_path = format!("{}/.rjsh_history", home_dir);
    if rl.load_history(&history_path).is_err() {
        std::fs::File::create(&history_path)?;
    }

    let mut shell = rjsh::shell::DefaultShell::default();

    loop {
        let readline = rl.readline("$ ");
        match readline {
            Ok(line) => {
                if line.trim() == "" {
                    continue;
                }
                let mut parser = Parser::new(line.clone());
                if let Some(command) = parser.parse_command() {
                    if command.name != "" {
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
    Ok(())
}
