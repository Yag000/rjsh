use std::{
    os::unix::process::ExitStatusExt,
    process::{Child, ExitStatus},
};

use super::{Process, ProcessId, Status};

#[derive(Debug)]
pub struct ExternalProcesss {
    name: String,
    pid: ProcessId,
    status: Status,
    exit_status: Option<ExitStatus>,

    child: Child,
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
        self.exit_status = exit_status;
        self.status = Status::from(exit_status);
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
