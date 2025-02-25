use std::{
    ffi::CString,
    fs::OpenOptions,
    os::fd::{IntoRawFd, RawFd},
    process::exit,
};

use nix::unistd::{dup2, execvp, fork, getpid, setpgid, ForkResult, Pid};

use crate::{
    builtins::get_builtin,
    error::UnwrapPrintError,
    parser::ast::{Redirectee, Redirection, RedirectionPermission, RedirectionType},
    proc::{
        job::{Job, Pgid},
        ExternalProcesss, InternalProcess, ProcessId, Status,
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

fn prepare_child(ast: &crate::parser::ast::Command, pgid: Pgid) {
    let mut redirections = RedirectionHolder::default();
    ast.redirections.iter().for_each(|r| {
        redirections.update(r);
    });

    if let Err(e) = redirections.dup_redirections() {
        eprintln!("rjsh: {e}");
        exit(1);
    }

    if let Err(e) = setpgid(getpid(), Pid::from_raw(pgid.0)) {
        eprintln!("rjsh: {e}");
        exit(1);
    }
}

enum RjshForkResult {
    Child(ProcessId),
    Exit(i32),
}

fn fork_execute(
    shell: &mut dyn Shell,
    ast: crate::parser::ast::Command,
) -> anyhow::Result<RjshForkResult> {
    if !ast.background {
        if let Some(builtin) = get_builtin(&ast) {
            let exit_code = builtin.call(shell, &ast.args).unwrap_error_with_print();
            return Ok(RjshForkResult::Exit(exit_code));
        }
    }

    let fork_result = unsafe { fork()? };

    if let ForkResult::Parent { child } = fork_result {
        return Ok(RjshForkResult::Child(ProcessId(child.as_raw())));
    }

    prepare_child(&ast, Pgid(0));

    if let Some(builtin) = get_builtin(&ast) {
        let exit_code = builtin.call(shell, &ast.args).unwrap_error_with_print();
        exit(exit_code);
    }

    // Don't forget to add the command name to the args
    let mut args = vec![ast.name.clone()];
    args.extend(ast.args);

    let c_args: Vec<CString> = args
        .iter()
        .map(|arg| CString::new(arg.clone()).unwrap())
        .collect();

    let c_name = CString::new(ast.name).unwrap();

    let Err(e) = execvp(c_name.as_ref(), c_args.as_ref());

    eprintln!("rjsh: {e}");

    exit(1);
}

fn ast_to_job(shell: &mut dyn Shell, ast: crate::parser::ast::Command) -> anyhow::Result<Job> {
    let background = ast.background;
    let name = ast.to_string();

    match fork_execute(shell, ast)? {
        RjshForkResult::Child(child_pid) => {
            let process = ExternalProcesss::new(child_pid, name.clone());
            Ok(Job::new(
                Pgid(child_pid.0),
                vec![Box::new(process)],
                Status::Running,
                background,
                name,
            ))
        }
        // Better handle this. The job is not properly printed etc...
        RjshForkResult::Exit(code) => {
            let process = InternalProcess::new(name.clone(), code);
            Ok(Job::new(
                Pgid(0),
                vec![Box::new(process)],
                Status::Done,
                background,
                name,
            ))
        }
    }
}

pub fn execute_command(
    shell: &mut dyn Shell,
    command: crate::parser::ast::Command,
) -> anyhow::Result<Option<i32>> {
    let background = command.background;
    let mut job = ast_to_job(shell, command)?;

    job.update(!background)?;
    match job.last_status {
        Status::Done | Status::Killed => {
            let code = job
                .exit_status()
                .expect("rjsh: wow, that should not happen")
                .code()
                .expect("rjsh: wow, that should not happen again");

            std::env::set_var("?", code.to_string());
            Ok(Some(code))
        }
        Status::Running | Status::Stopped => {
            shell.add_job(job);
            Ok(None)
        }
    }
}
