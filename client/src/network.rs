use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ServerMessage {
    Error { detail: String },
    JobUpdate { job: JobUpdateData },
    MeshGenerated { mesh: serde_json::Value, skeleton: serde_json::Value, clip: serde_json::Value },
    MotionGenerated { clip: serde_json::Value },
}

#[derive(Deserialize, Debug)]
pub struct JobUpdateData {
    pub id: String,
    pub job_type: String,
    pub status: String,
    pub progress: f64,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JobRequest {
    pub job_type: String,
    pub params: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientMessage {
    pub job_request: Option<JobRequest>,
}
