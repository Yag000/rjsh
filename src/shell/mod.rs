use std::process::Command;

pub trait Shell {
    fn execute_command(&mut self, command: &crate::parser::ast::Command) -> anyhow::Result<()>;

    fn exit_code(&self) -> i32;
}

#[derive(Default, Debug)]
pub struct DefaultShell {
    exit_code: i32,
}

fn ast_to_command(ast: &crate::parser::ast::Command) -> Command {
    let mut cmd = Command::new(&ast.args[0]);
    cmd.args(&ast.args[1..]);
    cmd
}

impl Shell for DefaultShell {
    fn execute_command(&mut self, command: &crate::parser::ast::Command) -> anyhow::Result<()> {
        let mut cmd = ast_to_command(command);
        let exit = cmd.status()?;
        self.exit_code = exit
            .code()
            .ok_or(anyhow::anyhow!("process exited with signal"))?;
        Ok(())
    }

    fn exit_code(&self) -> i32 {
        self.exit_code
    }
}
