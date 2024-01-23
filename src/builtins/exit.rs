use crate::shell::Shell;

use super::BuiltIn;

pub struct Exit {}

impl BuiltIn for Exit {
    fn call(&self, shell: &mut dyn Shell, args: &[String]) -> anyhow::Result<i32> {
        if args.len() > 1 {
            return Err(anyhow::anyhow!("too many arguments"));
        }
        let exit_code = if args.is_empty() {
            shell.last_exit_code()
        } else {
            args[0].parse::<i32>()?
        };
        shell.exit();
        Ok(exit_code)
    }
}
