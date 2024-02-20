use std::fmt::Display;

use super::{ExitStatus, Process, Status};

#[derive(Debug, Clone, Copy)]
pub struct Pgid(pub i32);

pub struct Job {
    pub id: usize,
    pub pgid: Pgid,
    leader: usize,
    pub background: bool,
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
        background: bool,
        name: String,
    ) -> Job {
        Job {
            id: 0,
            // Find a better way of dealing with this.
            // Maybe leader should be on the processes itself ?
            // Or an argument of this function, which is already quite big...
            leader: 0,
            pgid,
            processes,
            last_status,
            background,
            name,
        }
    }

    fn update_status(&mut self) {
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
    }

    pub fn update(&mut self, blocking: bool) -> Result<(), anyhow::Error> {
        for process in &mut self.processes {
            if process.status().is_finished() {
                continue;
            }
            process.wait(blocking)?;
        }

        let last_status = self.last_status;

        self.update_status();

        if last_status != self.last_status {
            // We should not print an update on a foreground job that is finished
            if self.background || !self.last_status.is_finished() {
                println!("{self}");
            }
        }

        Ok(())
    }

    pub fn exit_status(&self) -> Option<ExitStatus> {
        self.processes[self.leader].exit_status()
    }
}
