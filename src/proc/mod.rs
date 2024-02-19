use std::{os::unix::process::ExitStatusExt, process::Child};

use anyhow::anyhow;
use enum_stringify::EnumStringify;

pub mod job;
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ExitStatus {
    exit_status: ExitStatusEnum,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum ExitStatusEnum {
    Done(i32),
    Killed(KilledInfo),
    Stopped(i32),
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct KilledInfo {
    exit_code: i32,
    signal: i32,
}

impl ExitStatus {
    pub fn code(&self) -> Option<i32> {
        match &self.exit_status {
            ExitStatusEnum::Done(code) => Some(*code),
            ExitStatusEnum::Killed(info) => Some(info.exit_code),
            ExitStatusEnum::Stopped(_) => None,
        }
    }

    pub fn killed(&self) -> Option<i32> {
        match &self.exit_status {
            ExitStatusEnum::Done(_) => None,
            ExitStatusEnum::Killed(info) => Some(info.signal),
            ExitStatusEnum::Stopped(_) => None,
        }
    }

    pub fn stopped_signal(&self) -> Option<i32> {
        match &self.exit_status {
            ExitStatusEnum::Done(_) => None,
            ExitStatusEnum::Killed(_) => None,
            ExitStatusEnum::Stopped(code) => Some(*code),
        }
    }
}
impl TryFrom<std::process::ExitStatus> for ExitStatus {
    type Error = anyhow::Error;

    fn try_from(value: std::process::ExitStatus) -> Result<Self, Self::Error> {
        if let Some(code) = value.code() {
            Ok(ExitStatus {
                exit_status: ExitStatusEnum::Done(code),
            })
        } else if let Some(signal) = value.signal() {
            if let Some(code) = value.code() {
                Ok(ExitStatus {
                    exit_status: ExitStatusEnum::Killed(KilledInfo {
                        exit_code: code,
                        signal,
                    }),
                })
            } else {
                Err(anyhow!("No exit code"))
            }
        } else if let Some(code) = value.stopped_signal() {
            Ok(ExitStatus {
                exit_status: ExitStatusEnum::Stopped(code),
            })
        } else {
            Err(anyhow!("Unknown state"))
        }
    }
}
impl From<Option<ExitStatus>> for Status {
    fn from(value: Option<ExitStatus>) -> Self {
        match value {
            Some(status) => {
                if status.stopped_signal().is_some() {
                    Status::Stopped
                } else {
                    Status::Done
                }
            }
            None => Status::Running,
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

#[derive(Debug)]
pub struct ExternalProcesss {
    name: String,
    pid: ProcessId,
    status: Status,
    exit_status: Option<ExitStatus>,

    child: Child,
}

impl Process for ExternalProcesss {
    fn pid(&self) -> ProcessId {
        self.pid
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn status(&self) -> Status {
        self.status
    }

    fn exit_status(&self) -> Option<ExitStatus> {
        self.exit_status
    }

    fn wait(&mut self, blocking: bool) -> Result<Status, anyhow::Error> {
        let exit_status = if blocking {
            Some(self.child.wait()?)
        } else {
            self.child.try_wait()?
        };
        self.exit_status = exit_status.map(ExitStatus::try_from).transpose()?;
        self.status = Status::from(self.exit_status);
        Ok(self.status)
    }
}

impl ExternalProcesss {
    pub fn new(child: Child, name: String) -> Self {
        let pid = ProcessId(child.id());
        let status = Status::Running;
        ExternalProcesss {
            name,
            child,
            pid,
            status,
            exit_status: None,
        }
    }
}
