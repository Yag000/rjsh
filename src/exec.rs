use std::{
    ffi::CString,
    fs::OpenOptions,
    os::fd::{IntoRawFd, RawFd},
    process::exit,
};

use nix::unistd::{dup2, execvp, fork, ForkResult};

use crate::{
    error::UnwrapPrintError,
    parser::ast::{Redirectee, Redirection, RedirectionPermission, RedirectionType},
    proc::{
        job::{Job, Pgid},
        ExternalProcesss, ProcessId, Status,
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

    fn dup_redirections(self) -> anyhow::Result<()> {
        if let Some(fd) = self.stdin {
            dup2(0, fd)?;
        }
        if let Some(fd) = self.stdout {
            dup2(1, fd)?;
        }
        if let Some(fd) = self.stderr {
            dup2(2, fd)?;
        }
        Ok(())
    }
}

fn fork_execute(ast: crate::parser::ast::Command) -> anyhow::Result<ProcessId> {
    let fork_result = unsafe { fork()? };

    if let ForkResult::Parent { child } = fork_result {
        return Ok(ProcessId(child.as_raw()));
    }

    let mut redirections = RedirectionHolder::default();
    ast.redirections.iter().for_each(|r| {
        redirections.update(r);
    });

    if let Err(e) = redirections.dup_redirections() {
        eprintln!("rjsh: {e}");
        exit(1);
    }

    // Don't forget to add the command name to the args
    let mut args = vec![ast.name.clone()];
    args.extend(ast.args);

    let c_args: Vec<CString> = args
        .iter()
        .map(|arg| CString::new(arg.clone()).unwrap())
        .collect();

    let c_name = CString::new(ast.name).unwrap();

    let res = execvp(c_name.as_ref(), c_args.as_ref());

    if let Err(e) = res {
        eprintln!("rjsh: {e}");
    }

    exit(1);
}

fn ast_to_job(ast: crate::parser::ast::Command) -> anyhow::Result<Job> {
    let background = ast.background;
    let name = ast.to_string();

    let child_pid = fork_execute(ast)?;
    let process = ExternalProcesss::new(child_pid, name.clone());

    Ok(Job::new(
        Pgid(child_pid.0),
        vec![Box::new(process)],
        Status::Running,
        background,
        name,
    ))
}

pub fn execute_command(
    shell: &mut dyn Shell,
    command: crate::parser::ast::Command,
) -> anyhow::Result<Option<i32>> {
    let mut exit_code = None;
    match crate::builtins::get_builtin(&command) {
        Some(builtin) => {
            exit_code = Some(builtin.call(shell, &command.args).unwrap_error_with_print());
        }
        None => {
            let background = command.background;
            let mut job = ast_to_job(command)?;

            job.update(!background)?;
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
