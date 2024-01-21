use std::{
    os::unix::process::ExitStatusExt,
    process::{Command, ExitStatus},
};

use crate::error::UnwrapPrintError;

pub trait Shell {
    fn execute_command(&mut self, command: &crate::parser::ast::Command) -> anyhow::Result<()>;

    fn exit_code(&self) -> i32;
}

#[derive(Default, Debug)]
pub struct DefaultShell {
    exit_code: i32,
}

fn ast_to_command(ast: &crate::parser::ast::Command) -> Command {
    let mut cmd = Command::new(&ast.name);
    cmd.args(&ast.args);
    cmd
}

impl Shell for DefaultShell {
    fn execute_command(&mut self, command: &crate::parser::ast::Command) -> anyhow::Result<()> {
        match crate::builtins::get_builtin(command) {
            Some(builtin) => {
                self.exit_code = builtin.call(&command.args).unwrap_error_with_print();
            }
            None => {
                let mut cmd = ast_to_command(command);
                // Handle NONE if it was stopped/killed by a signal
                let exit = match cmd.status() {
                    Ok(status) => status,
                    Err(err) => {
                        eprintln!("rjsh: {}", err);
                        ExitStatus::from_raw(1)
                    }
                };
                self.exit_code = match exit
                    // Handle NONE if it was stopped/killed by a signal
                    .code()
                {
                    Some(code) => code,
                    None => {
                        eprintln!("rjsh: terminated by signal");
                        1
                    }
                };
            }
        }
        Ok(())
    }

    fn exit_code(&self) -> i32 {
        self.exit_code
    }
}
