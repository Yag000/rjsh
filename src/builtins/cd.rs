use super::BuiltIn;

pub struct Cd {}

fn set_new_cwd(dir: &str) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    std::env::set_var("OLDPWD", cwd);
    std::env::set_current_dir(dir)?;
    Ok(())
}

impl BuiltIn for Cd {
    fn call(&self, args: &[String]) -> anyhow::Result<i32> {
        if args.len() > 1 {
            eprintln!("cd: too many arguments");
            return Ok(1);
        }
        if args.is_empty() {
            let home = std::env::var("HOME")?;
            set_new_cwd(&home)?;
        } else {
            let path = args[0].clone();
            if path == "-" {
                let oldpwd = std::env::var("OLDPWD")?;
                set_new_cwd(&oldpwd)?;
            } else {
                set_new_cwd(&path)?;
            }
        }
        Ok(0)
    }
}
