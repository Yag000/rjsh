use std::{
    fs::OpenOptions,
    os::fd::FromRawFd,
    process::{Command, Stdio},
};

use crate::{
    error::UnwrapPrintError,
    parser::ast::{Redirectee, Redirection, RedirectionPermission, RedirectionType},
    proc::{external_process::ExternalProcesss, Job, Pgid, Process, Status},
    shell::Shell,
};

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
            let mut cmd = ast_to_command(command);
            let child = cmd.spawn()?;
            let mut process = ExternalProcesss::new(child, command.to_string());

            let exit = process.wait(!command.background)?;
            //TODO: Handle NONE if it was stopped/killed by a signal
            match exit {
                Status::Done | Status::Killed => {
                    exit_code = Some(
                        process
                            .exit_status()
                            .expect("rjsh: wow, that should not happen")
                            .code()
                            .expect("rjsh: wow, that should not happen again"),
                    );
                }
                status => {
                    let job = Job::new(
                        Pgid(i32::try_from(process.pid().0)?),
                        vec![Box::new(process)],
                        status,
                        command.to_string(),
                    );
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
