use std::cmp::max;

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
        let num_machines = self.number_of_machines();
        let num_jobs = self.number_of_jobs();
        let mut end_time: usize = 0;
        let mut machine_times = vec![0; num_machines];
        let mut job_times = vec![0; num_jobs];
        let mut job_operation_numbers = vec![1; num_jobs];
        for job_number in sequence {
            let job = self.job(*job_number);
            let operation_number = job_operation_numbers[job.number - 1];
            let operation = &job.operations[operation_number - 1];
            // Update next operation for job
            job_operation_numbers[job.number - 1] = operation_number + 1;
            let machine = operation.machine;

            // Start time must be after time and when job and machine is ready
            let start_time = max(machine_times[machine], job_times[job.number - 1]);
            // Update when machine and job is ready for a new operation
            machine_times[machine] = start_time + operation.time;
            job_times[job.number - 1] = start_time + operation.time;
            // Update the latest end time
            end_time = max(
                max(machine_times[machine], job_times[job.number - 1]),
                end_time,
            );
        }
        end_time
    }
}
