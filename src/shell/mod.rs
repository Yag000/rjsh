use crate::proc::{job_table::JobTable, Job};

pub trait Shell {
    fn add_job(&mut self, job: Job);

    fn last_exit_code(&self) -> i32;

    fn exit(&mut self);

    fn should_exit(&self) -> bool;

    fn job_number(&self) -> usize;

    fn update_jobs(&mut self);

    fn print_jobs(&self);

    fn get_job_pid(&self, job_id: usize) -> anyhow::Result<i32>;
}

#[derive(Default)]
pub struct DefaultShell {
    last_exit_code: i32,
    should_exit: bool,

    job_table: JobTable,
}

impl Shell for DefaultShell {
    fn add_job(&mut self, job: Job) {
        self.job_table.add_job(job);
    }

    fn last_exit_code(&self) -> i32 {
        self.last_exit_code
    }

    fn exit(&mut self) {
        self.should_exit = true;
    }

    fn should_exit(&self) -> bool {
        self.should_exit
    }

    fn job_number(&self) -> usize {
        self.job_table.size()
    }

    fn update_jobs(&mut self) {
        if let Err(e) = self.job_table.update() {
            eprintln!("rjsh: {e}");
        }
    }

    fn print_jobs(&self) {
        self.job_table.print_jobs();
    }

    fn get_job_pid(&self, job_id: usize) -> anyhow::Result<i32> {
        Ok(self
            .job_table
            .get_job(job_id)
            .ok_or_else(|| anyhow::anyhow!("Job not found"))?
            .pgid
            .0)
    }
}
