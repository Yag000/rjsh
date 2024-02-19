use std::{
    fs::OpenOptions,
    os::fd::{FromRawFd, IntoRawFd, RawFd},
    process::{Command, Stdio},
};

use crate::{
    error::UnwrapPrintError,
    parser::ast::{Redirectee, Redirection, RedirectionPermission, RedirectionType},
    proc::{
        job::{Job, Pgid},
        ExternalProcesss, Process, Status,
    },
    shell::Shell,
};

#[derive(Default)]
struct RedirectionHolder {
    stdin: Option<RawFd>,
    stdout: Option<RawFd>,
    stderr: Option<RawFd>,
}

impl RedirectionHolder {
    fn update(&mut self, redirection: &Redirection) {
        match redirection.type_ {
            RedirectionType::Stdin => self.stdin = Some(Self::redirection_to_raw_fd(redirection)),
            RedirectionType::Stdout => self.stdout = Some(Self::redirection_to_raw_fd(redirection)),
            RedirectionType::Stderr => self.stderr = Some(Self::redirection_to_raw_fd(redirection)),
        }
    }

    fn redirection_to_raw_fd(redirection: &Redirection) -> RawFd {
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
                file.into_raw_fd()
            }
            Redirectee::FileDescriptor(fd) => fd,
        }
    }

    fn update_command(&self, cmd: &mut Command) {
        if let Some(fd) = self.stdin {
            cmd.stdin(unsafe { Stdio::from_raw_fd(fd) });
        }
        if let Some(fd) = self.stdout {
            cmd.stdout(unsafe { Stdio::from_raw_fd(fd) });
        }
        if let Some(fd) = self.stderr {
            cmd.stderr(unsafe { Stdio::from_raw_fd(fd) });
        }
    }
}

fn ast_to_job(ast: &crate::parser::ast::Command) -> anyhow::Result<Job> {
    let mut cmd = Command::new(&ast.name);
    cmd.args(&ast.args);

    let mut redirections = RedirectionHolder::default();
    ast.redirections.iter().for_each(|r| {
        redirections.update(r);
    });

    redirections.update_command(&mut cmd);

    let child = cmd.spawn()?;
    let process = ExternalProcesss::new(child, ast.to_string());

    Ok(Job::new(
        Pgid(i32::try_from(process.pid().0)?),
        vec![Box::new(process)],
        Status::Running,
        ast.background,
        ast.to_string(),
    ))
}

pub fn execute_command(
    shell: &mut dyn Shell,
    command: &crate::parser::ast::Command,
) -> anyhow::Result<Option<i32>> {
    let mut exit_code = None;
    match crate::builtins::get_builtin(command) {
        Some(builtin) => {
            exit_code = Some(builtin.call(shell, &command.args).unwrap_error_with_print());
        }
        None => {
            let mut job = ast_to_job(command)?;

            job.update(!command.background)?;
            //TODO: Handle NONE if it was stopped/killed by a signal
            match job.last_status {
                Status::Done | Status::Killed => {
                    exit_code = Some(
                        job.exit_status()
                            .expect("rjsh: wow, that should not happen")
                            .code()
                            .expect("rjsh: wow, that should not happen again"),
                    );
                }
                Status::Running | Status::Stopped => {
                    shell.add_job(job);
                }
            }
        }
    }
    if let Some(exit_code) = exit_code {
        std::env::set_var("?", exit_code.to_string());
    }
    Ok(exit_code)
}
