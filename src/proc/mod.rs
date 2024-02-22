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

impl From<ExitStatusEnum> for ExitStatus {
    fn from(exit_status: ExitStatusEnum) -> Self {
        ExitStatus { exit_status }
    }
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
struct ProcessStatus {
    status: Status,
    exit_status: Option<ExitStatus>,
}

impl Default for ProcessStatus {
    fn default() -> Self {
        ProcessStatus {
            status: Status::Running,
            exit_status: None,
        }
    }
}

impl ProcessStatus {
    pub fn update(&mut self, status: WaitStatus) {
        match status {
            WaitStatus::Exited(_, code) => {
                self.exit_status = Some(ExitStatusEnum::Done(code).into())
            }
            WaitStatus::Signaled(_, sig, _) => {
                self.exit_status = Some(ExitStatusEnum::Killed(sig as i32).into())
            }
            WaitStatus::Stopped(_, sig) | WaitStatus::PtraceEvent(_, sig, _) => {
                self.exit_status = Some(ExitStatusEnum::Stopped(sig as i32).into())
            }
            WaitStatus::PtraceSyscall(_) | WaitStatus::Continued(_) => self.exit_status = None,
            WaitStatus::StillAlive => {}
        };
        self.status = Status::from(self.exit_status);
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
    status: ProcessStatus,
}

impl Process for ExternalProcesss {
    fn pid(&self) -> ProcessId {
        self.pid
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn status(&self) -> Status {
        self.status.status
    }

    fn exit_status(&self) -> Option<ExitStatus> {
        self.status.exit_status
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

        self.status.update(wait_res);
        Ok(self.status.status)
    }
}

impl ExternalProcesss {
    pub fn new(pid: ProcessId, name: String) -> Self {
        let status = ProcessStatus::default();
        ExternalProcesss { name, pid, status }
    }
}

#[derive(Debug)]
pub struct InternalProcess {
    name: String,
    status: ProcessStatus,
}

impl Process for InternalProcess {
    fn pid(&self) -> ProcessId {
        panic!("Internal process does not have a pid");
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn status(&self) -> Status {
        self.status.status
    }

    fn exit_status(&self) -> Option<ExitStatus> {
        self.status.exit_status
    }

    fn wait(&mut self, _blocking: bool) -> Result<Status, anyhow::Error> {
        Ok(self.status.status)
    }
}

impl InternalProcess {
    pub fn new(name: String, exit_code: i32) -> Self {
        let status = ProcessStatus {
            status: Status::Done,
            exit_status: Some(ExitStatusEnum::Done(exit_code).into()),
        };
        InternalProcess { name, status }
    }
}
