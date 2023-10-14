use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use uuid::Uuid;
use serde::{Serialize, Deserialize};
use async_channel::{Sender, Receiver, unbounded};
use futures_util::stream::StreamExt;
use tokio::time::sleep;

use hex;
use methods::{VERIFY_IPFS_CONTENT_ELF, VERIFY_IPFS_CONTENT_ID};
use risc0_zkvm::{
    Receipt, serde::{to_vec, from_slice}, MemoryImage, Program, MEM_SIZE, PAGE_SIZE,

};


#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct Job {
    pub id: String,
    pub ipfs_hash: String,
    pub start: u64,
    pub end: u64,
    pub status: Arc<Mutex<JobStatus>>,
    pub result: Option<String>,
    pub created_at: Instant,
}

pub struct JobManager {
    jobs: Arc<Mutex<HashMap<String, Arc<Job>>>>,
    sender: Sender<String>,
    receiver: Receiver<String>,
}

impl JobManager {
    const MAX_WORKERS: usize = 4;
    const CLEANUP_INTERVAL: Duration = Duration::from_secs(30 * 60);

    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        Self {
            jobs: Arc::new(Mutex::new(HashMap::new())),
            sender,
            receiver,
        }
    }

    pub async fn register_job(&mut self, job: Job) {
        let job_id = job.id.clone();
        let mut jobs = self.jobs.lock().expect("Failed to acquire lock on jobs");
        jobs.insert(job_id.clone(), Arc::new(job));
        self.sender.send(job_id.clone()).await.unwrap();
        
    }

    pub async fn execute_job(&mut self, job_id: String) {
        let jobs = self.jobs.lock().unwrap();
        if let Some(job) = jobs.get(&job_id) {
            let mut status = job.status.lock().unwrap();
            *status = JobStatus::Running;
    
            let result = ipfs_host::functions::select_from_ipfs_generate_guest_input(
                &job.ipfs_hash.clone(), 
                job.start, 
                job.end,
            ).await;
    
            sleep(Duration::from_secs(5)).await;
            *status = JobStatus::Completed;
        }
    }

    pub async fn cleanup_jobs(&mut self) {
        let now = Instant::now();
        let mut jobs = self.jobs.lock().unwrap();
        jobs.retain(|_, job| {
            now.duration_since(job.created_at) < Self::CLEANUP_INTERVAL
        });
    }

    pub async fn get_job_status(&self, job_id: String) -> Option<JobStatus> {
        let jobs = self.jobs.lock().unwrap();
        jobs.get(&job_id).map(|job| Arc::clone(job).status.lock().unwrap().clone())
    }
}

pub async fn worker(manager: Arc<Mutex<JobManager>>) {
    let mut receiver = {
        let manager = manager.lock().unwrap();
        manager.receiver.clone()
    };

    while let Some(job) = receiver.next().await {
        let mut manager = manager.lock().unwrap();
        manager.execute_job(job).await;
    }
}

pub async fn cleanup_worker(manager: Arc<Mutex<JobManager>>) {
    loop {
        sleep(JobManager::CLEANUP_INTERVAL).await;
        let mut manager = manager.lock().unwrap();
        manager.cleanup_jobs().await;
    }
}

