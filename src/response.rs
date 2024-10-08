use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
pub struct GenericResponse {
    pub status: String,
    pub message: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Debug)]
pub struct FeedbackResponse {
    pub id: String,
    pub name: String,
    pub email: String,
    pub feedback: String,
    pub rating: f32,
    pub status: String,
    pub createdAt: DateTime<Utc>,
    pub updatedAt: DateTime<Utc>,
}

#[derive(Serialize, Debug)]
pub struct FeedbackData {
    pub feedback: FeedbackResponse,
}

#[derive(Serialize, Debug)]
pub struct SingleFeedbackResponse {
    pub status: &'static str,
    pub data: FeedbackData,
}

#[derive(Serialize, Debug)]
pub struct FeedbackListResponse {
    pub status: &'static str,
    pub results: usize,
    pub feedbacks: Vec<FeedbackResponse>,
}
