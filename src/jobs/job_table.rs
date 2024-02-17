use anyhow::anyhow;

use super::Job;

#[derive(Default)]
pub struct JobTable {
    table: Vec<Option<Job>>,
    size: usize,
}

impl JobTable {
    pub fn add_job(&mut self, mut job: Job) {
        if self.size >= self.table.len() {
            job.id = self.size + 1;
            self.table.push(Some(job));
        } else {
            for i in 0..self.table.len() {
                if self.table[i].is_none() {
                    job.id = i + 1;
                    self.table[i] = Some(job);
                    break;
                }
            }
            unreachable!()
        }
        self.size += 1;
    }

    pub fn remove_job(&mut self, id: usize) -> Result<(), anyhow::Error> {
        if id > self.table.len() && self.size != 0 {
            Err(anyhow!("rjsh: Job index out of bounds"))
        } else {
            self.table[id - 1] = None;
            self.size -= 1;
            Ok(())
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn update(&mut self) -> Result<(), anyhow::Error> {
        let mut to_remove = Vec::new();
        for job in &mut self.table {
            if let Some(job) = job {
                job.update(false)?;
                if job.last_status.is_finished() {
                    to_remove.push(job.id)
                }
            }
        }

        for id in to_remove {
            self.remove_job(id)?;
        }

        Ok(())
    }

    pub fn print_jobs(&self) {
        for job in &self.table {
            if let Some(job) = job {
                println!("{job}");
            }
        }
    }
}
