use rjsh::lexer::Lexer;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

fn main() -> anyhow::Result<()> {
    let mut rl = DefaultEditor::new()?;

    let home_dir = std::env::var("HOME")?;
    let history_path = format!("{}/.rjsh_history", home_dir);
    if rl.load_history(&history_path).is_err() {
        std::fs::File::create(&history_path)?;
    }

    loop {
        let readline = rl.readline("$ ");
        match readline {
            Ok(line) => {
                let mut lexer = Lexer::new(line.clone());
                while let Some(token) = lexer.next_token() {
                    println!("Token: {}", token);
                }

                rl.add_history_entry(line.as_str())?;
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
