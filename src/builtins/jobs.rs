use crate::shell::Shell;

use super::BuiltIn;

pub struct Jobs {}

impl BuiltIn for Jobs {
    fn call(&self, shell: &mut dyn Shell, args: &[String]) -> anyhow::Result<i32> {
        if !args.is_empty() {
            return Err(anyhow::anyhow!("too many arguments"));
        }
        shell.print_jobs();
        Ok(0)
    }
}
