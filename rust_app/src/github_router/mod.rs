use std::sync::Arc;

use github::{GithubClient, SerializableContentItems};
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};

use crate::github;

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Message {
    message: String,
}

#[get("/simple_json")]
pub async fn simple_json() -> Json<Message> {
    Json(Message {
        message: "Python vs Rust".to_string(),
    })
}

#[get("/repos/numberOfRepos")]
pub async fn get_nb_repo(
    github_client: &State<Arc<GithubClient>>,
) -> Result<Json<usize>, status::Custom<String>> {
    github_client
        .get_number_of_repo()
        .await
        .map(Json)
        .map_err(|e| {
            error!("Failed to retrieve number of repository");
            status::Custom(Status::InternalServerError, e.to_string())
        })
}

#[get("/repos/list")]
pub async fn get_list_of_repos(
    github_client: &State<Arc<GithubClient>>,
) -> Result<Json<Vec<octocrab::models::Repository>>, status::Custom<String>> {
    info!("Fetching list of repos !");

    github_client
        .get_list_of_repos()
        .await
        .map(Json)
        .map_err(|e| {
            error!("Failed to fetch list of repos");
            status::Custom(Status::InternalServerError, e.to_string())
        })
}

#[get("/repos/<repo_name>")]
pub async fn get_repo(
    github_client: &State<Arc<GithubClient>>,
    repo_name: &str,
) -> Result<Json<octocrab::models::Repository>, status::Custom<String>> {
    info!("Repo name is {}", repo_name);

    github_client
        .get_repo(repo_name)
        .await
        .map(Json)
        .map_err(|e| {
            error!("Failed to retrieve repository '{}'", repo_name);
            status::Custom(Status::InternalServerError, e.to_string())
        })
}

#[get("/repos/<repo_name>/contents/wrong/<path>")]
pub async fn get_repo_contents(
    github_client: &State<Arc<GithubClient>>,
    repo_name: &str,
    path: &str,
) -> Result<Json<SerializableContentItems>, status::Custom<String>> {
    github_client
        .get_contents(repo_name, path)
        .await
        .map(Json)
        .map_err(|e| {
            error!("Failed to retrieve contents for {} and {}", repo_name, path);
            status::Custom(Status::InternalServerError, e.to_string())
        })
}

#[get("/repos/<repo_name>/contents/<path>")]
pub async fn get_repo_all_contents(
    github_client: &State<Arc<GithubClient>>,
    repo_name: &str,
    path: &str,
) -> Result<Json<SerializableContentItems>, status::Custom<String>> {
    github_client
        .get_contents_for_repo(repo_name, path)
        .await
        .map(Json)
        .map_err(|e| {
            error!("Failed to retrieve contents for {}", repo_name);
            status::Custom(Status::InternalServerError, e.to_string())
        })
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PayloadContent {
    repo_name: String,
    path: String,
}

#[post("/repos/contents", data = "<payload>")]
pub async fn get_config_from_stack_and_file(
    github_client: &State<Arc<GithubClient>>,
    payload: Json<PayloadContent>,
) -> Result<Json<SerializableContentItems>, Status> {
    match github_client
        .get_config_from_stack_and_file(&payload.repo_name, &payload.path)
        .await
    {
        Ok(content) => Ok(Json(content)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/repos/contents/string", data = "<payload>")]
pub async fn get_config_from_stack_and_file_string(
    github_client: &State<Arc<GithubClient>>,
    payload: Json<PayloadContent>,
) -> Result<Json<String>, Status> {
    match github_client
        .get_config_from_stack_and_file_string(&payload.repo_name, &payload.path)
        .await
    {
        Ok(content) => Ok(Json(content)),
        Err(_) => Err(Status::InternalServerError),
    }
}
