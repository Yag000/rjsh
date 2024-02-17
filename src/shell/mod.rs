use std::{
    fs::OpenOptions,
    os::unix::io::FromRawFd,
    process::{Command, Stdio},
};

use crate::{
    error::UnwrapPrintError,
    jobs::{external_process::ExternalProcesss, job_table::JobTable, Job, Pgid, Process, Status},
    parser::ast::{Redirectee, Redirection, RedirectionPermission, RedirectionType},
};

pub trait Shell {
    fn execute_command(&mut self, command: &crate::parser::ast::Command) -> anyhow::Result<()>;

    fn last_exit_code(&self) -> i32;

    fn exit(&mut self);

    fn should_exit(&self) -> bool;

    fn job_number(&self) -> usize;

    fn update_jobs(&mut self);

    fn print_jobs(&self);
}

#[derive(Default)]
pub struct DefaultShell {
    last_exit_code: i32,
    should_exit: bool,

    job_table: JobTable,
}

fn redirection_to_raw_fd(redirection: &Redirection) -> Stdio {
    match redirection.redirectee.clone() {
        Redirectee::FileName(path) => {
            let file = OpenOptions::new()
                .create(redirection.type_ != RedirectionType::Stdin)
                .write(redirection.type_ != RedirectionType::Stdin)
                .read(redirection.type_ == RedirectionType::Stdin)
                .truncate(redirection.permissions == RedirectionPermission::Truncate)
                .append(redirection.permissions == RedirectionPermission::Append)
                .open(path)
                .unwrap();
            Stdio::from(file)
        }
        Redirectee::FileDescriptor(fd) => unsafe { Stdio::from_raw_fd(fd) },
    }
}

fn ast_to_command(ast: &crate::parser::ast::Command) -> Command {
    let mut cmd = Command::new(&ast.name);
    cmd.args(&ast.args);
    ast.redirections.iter().fold(&mut cmd, |acc, r| {
        let file = redirection_to_raw_fd(r);
        match r.type_ {
            RedirectionType::Stdin => acc.stdin(file),
            RedirectionType::Stdout => acc.stdout(file),
            RedirectionType::Stderr => acc.stderr(file),
        }
    });
    cmd
}

impl Shell for DefaultShell {
    fn execute_command(&mut self, command: &crate::parser::ast::Command) -> anyhow::Result<()> {
        match crate::builtins::get_builtin(command) {
            Some(builtin) => {
                self.last_exit_code = builtin.call(self, &command.args).unwrap_error_with_print();
            }
            None => {
                let mut cmd = ast_to_command(command);
                //TODO: Handle NONE if it was stopped/killed by a signal
                let child = cmd.spawn()?;
                let mut process = ExternalProcesss::new(child, command.to_string());

                let exit = process.wait(!command.background)?;
                //TODO: Handle NONE if it was stopped/killed by a signal
                match exit {
                    Status::Done | Status::Killed => {
                        let exit_code = process
                            .exit_status()
                            .expect("rjsh: wow, that should not happen");
                        self.last_exit_code = exit_code
                            .code()
                            .expect("rjsh: wow, that should not happen again");
                    }
                    status => {
                        let job = Job::new(
                            Pgid(i32::try_from(process.pid().0)?),
                            vec![Box::new(process)],
                            status,
                            command.to_string(),
                        );
                        self.job_table.add_job(job);
                    }
                }
            }
        }
        std::env::set_var("?", self.last_exit_code.to_string());
        Ok(())
    }

    fn last_exit_code(&self) -> i32 {
        self.last_exit_code
    }

    fn exit(&mut self) {
        self.should_exit = true;
    }

    fn should_exit(&self) -> bool {
        self.should_exit
    }

    fn job_number(&self) -> usize {
        self.job_table.size()
    }

    fn update_jobs(&mut self) {
        if let Err(e) = self.job_table.update() {
            eprint!("rjsh: {e}");
        }
    }

    fn print_jobs(&self) {
        self.job_table.print_jobs();
    }
}
