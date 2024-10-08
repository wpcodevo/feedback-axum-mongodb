use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Default)]
pub struct FilterOptions {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

// #[derive(Deserialize, Debug)]
// pub struct ParamOptions {
//     pub id: String,
// }

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateFeedbackSchema {
    pub name: String,
    pub email: String,
    pub feedback: String,
    pub rating: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateFeedbackSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}
