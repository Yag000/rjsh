use std::{str::FromStr, usize};

use nix::sys::signal::Signal::{self, SIGTERM};

use crate::shell::Shell;

use super::BuiltIn;

pub struct Kill {}

impl BuiltIn for Kill {
    fn call(&self, shell: &mut dyn Shell, args: &[String]) -> anyhow::Result<i32> {
        if args.is_empty() {
            return Err(anyhow::anyhow!("not enough arguments"));
        }
        if args.len() > 2 {
            return Err(anyhow::anyhow!("too many arguments"));
        }

        let signal = if args.len() == 2 {
            Signal::try_from(i32::from_str(&args[0][1..])?)?
        } else {
            SIGTERM
        };

        let pid_s = if args.len() == 2 {
            args[1].clone()
        } else {
            args[0].clone()
        };

        if let Some(stripped) = pid_s.strip_prefix('%') {
            let job_id = stripped.parse::<usize>()?;
            let pgid = shell.get_job_pgid(job_id)?;
            nix::sys::signal::killpg(nix::unistd::Pid::from_raw(pgid), signal)?;
        } else {
            let pid = pid_s.parse::<i32>()?;
            nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid), signal)?;
        };

        Ok(0)
    }
}
