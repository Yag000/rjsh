use rjsh::editor::RjshEditor;
use rjsh::exec::execute_command;
use rjsh::parser::parse_command;
use rjsh::prompt::get_prompt;
use rjsh::shell::Shell;
use rustyline::error::ReadlineError;

fn main() -> anyhow::Result<()> {
    let mut rl = RjshEditor::new()?;

    let home_dir = std::env::var("HOME")?;
    let history_path = format!("{home_dir}/.rjsh_history");
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
                match parse_command(line.as_str()) {
                    Ok(command) => {
                        if !command.name.is_empty() {
                            let name = command.name.clone();

                            match execute_command(&mut shell, command) {
                                Ok(_) => {}
                                Err(e) => {
                                    eprintln!("rjsh: {e}");
                                }
                            }

                            if name != "exit" {
                                rl.add_history_entry(line)?;
                            }
                        }
                    }
                    Err(e) => eprintln!("{e}"),
                }
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("rjsh: {err:?}");
                break;
            }
        }
    }

    rl.save_history(&history_path)?;

    std::process::exit(shell.last_exit_code());
}
