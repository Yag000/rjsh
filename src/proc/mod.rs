use enum_stringify::EnumStringify;
use nix::{
    sys::wait::{waitpid, WaitPidFlag, WaitStatus},
    unistd::Pid,
};

pub mod job;
pub mod job_table;

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
    Killed(i32),
    Stopped(i32),
}

impl ExitStatus {
    pub fn code(&self) -> Option<i32> {
        match &self.exit_status {
            ExitStatusEnum::Done(code) => Some(*code),
            ExitStatusEnum::Killed(_) => None,
            ExitStatusEnum::Stopped(_) => None,
        }
    }

    pub fn killed(&self) -> Option<i32> {
        match &self.exit_status {
            ExitStatusEnum::Done(_) => None,
            ExitStatusEnum::Killed(sig) => Some(*sig),
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

    pub fn from_wait_status(status: WaitStatus) -> Option<Self> {
        match status {
            WaitStatus::Exited(_, code) => Some(ExitStatus {
                exit_status: ExitStatusEnum::Done(code),
            }),
            WaitStatus::Signaled(_, sig, _) => Some(ExitStatus {
                exit_status: ExitStatusEnum::Killed(sig as i32),
            }),
            WaitStatus::Stopped(_, sig) | WaitStatus::PtraceEvent(_, sig, _) => Some(ExitStatus {
                exit_status: ExitStatusEnum::Stopped(sig as i32),
            }),
            WaitStatus::PtraceSyscall(_) | WaitStatus::Continued(_) | WaitStatus::StillAlive => {
                None
            }
        }
    }
}

impl From<Option<ExitStatus>> for Status {
    fn from(value: Option<ExitStatus>) -> Self {
        match value {
            Some(status) => {
                if status.stopped_signal().is_some() {
                    Status::Stopped
                } else if status.killed().is_some() {
                    Status::Killed
                } else {
                    Status::Done
                }
            }
            None => Status::Running,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ProcessId(pub i32);

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
        let flags = if blocking {
            WaitPidFlag::WUNTRACED
        } else {
            WaitPidFlag::WNOHANG
                .union(WaitPidFlag::WUNTRACED)
                .union(WaitPidFlag::WCONTINUED)
        };
        let wait_res = waitpid(Pid::from_raw(self.pid.0), Some(flags))?;

        self.exit_status = ExitStatus::from_wait_status(wait_res);
        self.status = Status::from(self.exit_status);
        Ok(self.status)
    }
}

impl ExternalProcesss {
    pub fn new(pid: ProcessId, name: String) -> Self {
        let status = Status::Running;
        ExternalProcesss {
            name,
            pid,
            status,
            exit_status: None,
        }
    }
}
