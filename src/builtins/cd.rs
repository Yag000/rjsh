use crate::shell::Shell;

use super::BuiltIn;

pub struct Cd {}

fn set_new_cwd(dir: &str) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    std::env::set_var("OLDPWD", cwd);
    std::env::set_current_dir(dir)?;
    Ok(())
}

impl BuiltIn for Cd {
    fn call(&self, _: &mut dyn Shell, args: &[String]) -> anyhow::Result<i32> {
        if args.len() > 1 {
            return Err(anyhow::anyhow!("too many arguments"));
        }
        if args.is_empty() {
            let home = std::env::var("HOME")?;
            set_new_cwd(&home)?;
        } else {
            let path = args[0].clone();

            let new_cdw = if path == "-" {
                std::env::var("OLDPWD")?
            } else {
                path
            };

            set_new_cwd(&new_cdw)?;
        }
        Ok(0)
    }
}
