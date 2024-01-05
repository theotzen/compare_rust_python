use bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct DiffBaseSchema {
    pub stack_a: String,
    pub stack_b: String,
    pub file: String,
    pub left_not_right: Vec<String>,
    pub right_not_left: Vec<String>,
    pub same_key_diff_value: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileDiff {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub stack_a: String,
    pub stack_b: String,
    pub file: String,
    pub left_not_right: Vec<String>,
    pub right_not_left: Vec<String>,
    pub same_key_diff_value: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetOneDiffResponse {
    pub diff: FileDiff,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetAllDiffsResponse {
    pub stack_a: String,
    pub stack_b: String,
    pub files_with_diff: Vec<FileDiff>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetAllDiffsNoStackResponse {
    pub files_with_diff: Vec<FileDiff>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetAllDiffsPayload {
    pub stack_a: String,
    pub stack_b: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetLatestDiffResponse {
    pub stack_a: String,
    pub stack_b: String,
    pub diff: FileDiff,
    pub latest_diff: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetConfigsFromStacksPayload {
    pub stack_a: String,
    pub stack_b: String,
    pub file: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetConfigsFromStacksResponse {
    pub stack_a: String,
    pub stack_b: String,
    pub file: String,
    pub config_a: String,
    pub config_b: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ComputeAllDiffPayload {
    pub stack_a: String,
    pub stack_b: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ComputeAllDiffResponse {
    pub stack_a: String,
    pub stack_b: String,
    pub files_with_diff: Vec<FileDiff>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ToggleReviewPayload {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ToggleReviewResponse {
    pub status: String,
}
