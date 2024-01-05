use std::str::FromStr;
use std::sync::Arc;
use std::time::SystemTime;

use bson::doc;
use bson::oid::ObjectId;
use chrono::Utc;
use github::GithubClient;
use rocket::http::Status;
use rocket::response::{self, Responder, Response};
use rocket::serde::json::Json;
use rocket::Request;
use rocket::State;

use super::models;
use crate::db::DiffCollection;
use crate::github::{self, ConfigError, SerializableContent};
use crate::utils::compare_yaml_strings;

pub struct HttpCustomError {
    status: Status,
    message: String,
}

pub struct AppConfig {
    pub folder_a: String,
    pub folder_b: String,
}

impl<'r, 'o: 'r> Responder<'r, 'o> for HttpCustomError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'o> {
        Response::build()
            .status(self.status)
            .sized_body(self.message.len(), std::io::Cursor::new(self.message))
            .ok()
    }
}

#[get("/getOneDiffById/<diff_id>")]
pub async fn get_diff_by_id(
    diff_id: &str,
    db: &State<DiffCollection>,
) -> Result<Json<models::GetOneDiffResponse>, HttpCustomError> {
    let object_id = match ObjectId::from_str(&diff_id) {
        Ok(oid) => oid,
        Err(e) => {
            return Err(HttpCustomError {
                status: Status::BadRequest,
                message: e.to_string(),
            })
        }
    };

    match db.find_diff_by_id(object_id).await {
        Ok(file_diff) => Ok(Json(models::GetOneDiffResponse { diff: file_diff })),
        Err(e) => Err(HttpCustomError {
            status: Status::NotFound,
            message: e.to_string(),
        }),
    }
}

#[post("/insertOneDiff", data = "<payload>")]
pub async fn insert_diff(
    payload: Json<models::DiffBaseSchema>,
    db: &State<DiffCollection>,
) -> Result<Json<models::FileDiff>, HttpCustomError> {
    let payload = payload.into_inner();

    let inserted_result = db
        .insert_diff_from_diff_base_schema(payload)
        .await
        .map_err(|e| HttpCustomError {
            status: Status::InternalServerError,
            message: e.to_string(),
        })?;

    let inserted_id = inserted_result
        .inserted_id
        .as_object_id()
        .ok_or(HttpCustomError {
            status: Status::InternalServerError,
            message: "Can't deserialize id".to_string(),
        })?;

    db.find_diff_by_id(inserted_id)
        .await
        .map(Json)
        .map_err(|e| HttpCustomError {
            status: Status::InternalServerError,
            message: e.to_string(),
        })
}

#[post("/getLatestDiffsFromStacks", data = "<payload>")]
pub async fn get_latest_diffs_from_stacks(
    payload: Json<models::GetAllDiffsPayload>,
    mongo: &State<DiffCollection>,
) -> Result<Json<models::GetAllDiffsResponse>, HttpCustomError> {
    let payload = payload.into_inner();

    let mut diffs: Vec<models::FileDiff> = mongo
        .get_all_diffs_from_stacks(&payload)
        .await
        .map_err(|e| HttpCustomError {
            status: Status::NotFound,
            message: e.to_string(),
        })?;

    if diffs.is_empty() {
        error!(
            "No comparison found with {} and {}",
            &payload.stack_a, &payload.stack_b
        );
        return Err(HttpCustomError {
            status: Status::NotFound,
            message: "Couldn't get diff for these stacks".to_string(),
        });
    }

    diffs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    let latest_date = diffs.first().unwrap().created_at.unwrap();

    let latest_diffs = diffs
        .into_iter()
        .filter(|diff| diff.created_at == Some(latest_date))
        .collect();

    Ok(Json(models::GetAllDiffsResponse {
        stack_a: payload.stack_a,
        stack_b: payload.stack_b,
        files_with_diff: latest_diffs,
    }))
}

#[post("/getAllDiffsFromStacks", data = "<payload>")]
pub async fn get_all_diffs_from_stacks(
    payload: Json<models::GetAllDiffsPayload>,
    mongo: &State<DiffCollection>,
) -> Result<Json<models::GetAllDiffsResponse>, HttpCustomError> {
    let payload_unjson = payload.into_inner();
    mongo
        .get_all_diffs_from_stacks(&payload_unjson)
        .await
        .map(|s| models::GetAllDiffsResponse {
            stack_a: payload_unjson.stack_a,
            stack_b: payload_unjson.stack_b,
            files_with_diff: s,
        })
        .map(Json)
        .map_err(|e| HttpCustomError {
            status: Status::NotFound,
            message: e.to_string(),
        })
}

#[post("/getConfigsFromStacks", data = "<payload>")]
pub async fn get_configs_from_stacks_name(
    payload: Json<models::GetConfigsFromStacksPayload>,
    github_client: &State<Arc<GithubClient>>,
) -> Result<Json<models::GetConfigsFromStacksResponse>, HttpCustomError> {
    let payload = payload.into_inner();

    let config_a = github_client
        .get_config_from_stack_and_file_string(&payload.stack_a, &payload.file)
        .await
        .map_err(|e| HttpCustomError {
            status: Status::NotFound,
            message: e.to_string(),
        })?;

    let config_b = github_client
        .get_config_from_stack_and_file_string(&payload.stack_b, &payload.file)
        .await
        .map_err(|e| HttpCustomError {
            status: Status::NotFound,
            message: e.to_string(),
        })?;

    let response = models::GetConfigsFromStacksResponse {
        stack_a: payload.stack_a,
        stack_b: payload.stack_b,
        file: payload.file,
        config_a,
        config_b,
    };

    Ok(Json(response))
}

#[post("/computeAllDiffs", data = "<payload>")]
pub async fn compute_diff_for_all_files(
    payload: Json<models::ComputeAllDiffPayload>,
    github_client: &State<Arc<GithubClient>>,
    app_config: &State<Arc<AppConfig>>,
    mongo: &State<DiffCollection>,
) -> Result<Json<models::ComputeAllDiffResponse>, HttpCustomError> {
    let now = Utc::now();
    let system_time: SystemTime = now.into();

    let mut file_diffs: Vec<models::FileDiff> = Vec::new();
    let payload = payload.into_inner();

    let subfolders_a = github_client
        .get_contents_for_repo(&payload.stack_a, &app_config.folder_a)
        .await
        .map_err(|e| HttpCustomError {
            status: Status::NotFound,
            message: e.to_string(),
        })?;
    let subfolders_b = github_client
        .get_contents_for_repo(&payload.stack_a, &app_config.folder_b)
        .await
        .map_err(|e| HttpCustomError {
            status: Status::NotFound,
            message: e.to_string(),
        })?;

    let mut subfolders = subfolders_a
        .items
        .into_iter()
        .filter(|content_item| content_item.r#type == "dir")
        .collect::<Vec<SerializableContent>>();

    subfolders.extend(
        subfolders_b
            .items
            .into_iter()
            .filter(|content_item| content_item.r#type == "dir")
            .collect::<Vec<SerializableContent>>(),
    );

    info!("{} folders config to check", subfolders.len());

    for subfolder in subfolders {
        info!("Getting config for file {}", &subfolder.path);
        let content_stack_a = github_client
            .get_config_from_stack_and_file_string(
                &payload.stack_a,
                &format!("{}/config-overrides.yml", &subfolder.path),
            )
            .await
            .map_err(|e| {
                error!(
                    "Couldn't get config {} for stack_a {}",
                    &subfolder.path, &payload.stack_a
                );
                HttpCustomError {
                    status: Status::NotFound,
                    message: e.to_string(),
                }
            })?;

        let content_stack_b = match github_client
            .get_config_from_stack_and_file_string(
                &payload.stack_b,
                &format!("{}/config-overrides.yml", &subfolder.path),
            )
            .await
        {
            Ok(content) => content,
            Err(ConfigError::OctocrabError(_)) => {
                info!(
                    "Couldn't find file {} for stack {}",
                    &subfolder.path, &payload.stack_b
                );
                let stack_a_clone = payload.stack_a.clone();
                let stack_b_clone = payload.stack_b.clone();
                let file_diff_no_content = models::FileDiff {
                    id: None,
                    stack_a: stack_a_clone,
                    stack_b: stack_b_clone,
                    file: subfolder.path,
                    left_not_right: vec!["/*".to_string()],
                    right_not_left: Vec::new(),
                    same_key_diff_value: Vec::new(),
                    reviewed: Some("false".to_string()),
                    created_at: Some(system_time.into()),
                    updated_at: Some(system_time.into()),
                };
                let _inserted_result =
                    mongo
                        .insert_diff(&file_diff_no_content)
                        .await
                        .map_err(|e| HttpCustomError {
                            status: Status::InternalServerError,
                            message: e.to_string(),
                        })?;
                file_diffs.push(file_diff_no_content);
                continue;
            }
            Err(e) => {
                info!(
                    "Couldn't find file {} for stack {}",
                    &subfolder.path,
                    &e.to_string()
                );
                return Err(HttpCustomError {
                    status: Status::InternalServerError,
                    message: e.to_string(),
                });
            }
        };

        let (left_not_right, right_not_left, same_key_same_value, same_key_diff_value) =
            compare_yaml_strings(&content_stack_a, &content_stack_b);

        info!("Computed yaml diff for file {}", &subfolder.path);

        if left_not_right.is_empty()
            && right_not_left.is_empty()
            && same_key_diff_value.is_empty()
            && !same_key_same_value.is_empty()
        {
            info!(
                "Same config for stack {} and stack {}, continuing",
                &payload.stack_a, &payload.stack_b
            );
            continue;
        }
        let stack_a_clone = payload.stack_a.clone();
        let stack_b_clone = payload.stack_b.clone();
        let file_diff_with_content = models::FileDiff {
            id: None,
            stack_a: stack_a_clone,
            stack_b: stack_b_clone,
            file: subfolder.path,
            left_not_right: left_not_right,
            right_not_left: right_not_left,
            same_key_diff_value: same_key_diff_value,
            reviewed: Some("false".to_string()),
            created_at: Some(system_time.into()),
            updated_at: Some(system_time.into()),
        };
        let _inserted_result = mongo
            .insert_diff(&file_diff_with_content)
            .await
            .map_err(|e| HttpCustomError {
                status: Status::InternalServerError,
                message: e.to_string(),
            })?;
        file_diffs.push(file_diff_with_content);
    }

    Ok(Json(models::ComputeAllDiffResponse {
        stack_a: payload.stack_a,
        stack_b: payload.stack_b,
        files_with_diff: file_diffs,
    }))
}

#[post("/toggleReview", data = "<payload>")]
pub async fn toggle_review_endpoint(
    payload: Json<models::ToggleReviewPayload>,
    mongo: &State<DiffCollection>,
) -> Result<Json<models::ToggleReviewResponse>, HttpCustomError> {
    let diff_id =
        bson::oid::ObjectId::from_str(&payload.into_inner().id).map_err(|e| HttpCustomError {
            status: Status::InternalServerError,
            message: e.to_string(),
        })?;

    mongo
        .toggle_review(diff_id)
        .await
        .map_err(|e| HttpCustomError {
            status: Status::InternalServerError,
            message: e.to_string(),
        })
        .map(|updated| {
            Json(models::ToggleReviewResponse {
                status: updated.matched_count.to_string(),
            })
        })
}
