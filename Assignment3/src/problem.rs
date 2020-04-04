use crate::parser::parse_file;

#[derive(Debug)]
pub struct Operation {
    pub machine: usize,
    pub time: usize,
    pub part_number: usize,
    pub job_number: usize,
}

#[derive(Debug)]
pub struct Job {
    pub operations: Vec<Operation>,
    pub number: usize,
}

#[derive(Debug)]
pub struct Problem {
    jobs: Vec<Job>,
    machines: usize,
}

impl Job {
    pub fn new(number: usize) -> Self {
        Self {
            operations: Vec::new(),
            number,
        }
    }

    pub fn add_operation(&mut self, operation: Operation) {
        self.operations.push(operation);
    }
}

impl Problem {
    pub fn from_file(path: &str) -> Self {
        let lines = parse_file(path);
        let num_machines: usize = lines[0][1];
        let mut jobs = Vec::new();
        for line_number in 1..lines.len() {
            let line = &lines[line_number];
            let mut job = Job::new(line_number);
            for i in (0..line.len()).step_by(2) {
                let operation = Operation {
                    machine: line[i],
                    time: line[i + 1],
                    part_number: job.operations.len() + 1,
                    job_number: line_number,
                };
                job.add_operation(operation);
            }
            jobs.push(job);
        }
        Problem {
            jobs,
            machines: num_machines,
        }
    }

    pub fn get_job_operations(&self) -> Vec<usize> {
        let mut job_operations: Vec<usize> = Vec::new();
        for (i, job) in self.jobs.iter().enumerate() {
            for _ in job.operations.iter() {
                job_operations.push(i);
            }
        }
        job_operations
    }

    pub fn number_of_machines(&self) -> usize {
        self.machines
    }

    pub fn number_of_jobs(&self) -> usize {
        self.jobs.len()
    }

    pub fn job(&self, index: usize) -> &Job {
        &self.jobs[index]
    }

    pub fn calc_fitness(&self, sequence: &Vec<usize>) -> usize {
        0
    }
}
