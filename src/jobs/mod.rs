use std::{fmt::Display, process::ExitStatus, usize};

use enum_stringify::EnumStringify;

pub mod external_process;
pub mod job_table;

#[derive(Debug, Clone, Copy)]
pub struct ProcessId(pub u32);

#[derive(EnumStringify, Clone, Copy, Debug, PartialEq)]
pub enum Status {
    Running,
    Killed,
    Stopped,
    Done,
}

impl Status {
    fn is_finished(&self) -> bool {
        match self {
            Self::Running | Self::Stopped => false,
            Self::Done | Self::Killed => true,
        }
    }
}

pub trait Process {
    fn pid(&self) -> ProcessId;
    fn name(&self) -> String;
    fn status(&self) -> Status;
    fn exit_status(&self) -> Option<ExitStatus>;
    fn wait(&mut self, blocking: bool) -> Result<Status, anyhow::Error>;
}

#[derive(Debug, Clone, Copy)]
pub struct Pgid(pub i32);

pub struct Job {
    pub id: usize,
    pub pgid: Pgid,
    pub last_status: Status,
    pub name: String,
    pub processes: Vec<Box<dyn Process>>,
}

impl Display for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]\t{}\t{}\t{}",
            self.id, self.pgid.0, self.last_status, self.name
        )
    }
}

impl Job {
    pub fn new(
        pgid: Pgid,
        processes: Vec<Box<dyn Process>>,
        last_status: Status,
        name: String,
    ) -> Job {
        Job {
            id: 0,
            pgid,
            processes,
            last_status,
            name,
        }
    }

    pub fn update(&mut self, blocking: bool) -> Result<(), anyhow::Error> {
        for process in &mut self.processes {
            if process.status().is_finished() {
                continue;
            }
            process.wait(blocking)?;
        }

        let last_status = self.last_status;

        if self.processes.iter().any(|p| p.status() == Status::Running) {
            self.last_status = Status::Running;
        } else if self.processes.iter().any(|p| p.status() == Status::Stopped) {
            self.last_status = Status::Stopped;
        } else {
            //TODO: Handle detached
            if self.processes.iter().any(|p| p.status() == Status::Killed) {
                self.last_status = Status::Killed;
            } else {
                self.last_status = Status::Done;
            }
        }

        if last_status != self.last_status {
            println!("{self}");
        }

        Ok(())
    }
}
