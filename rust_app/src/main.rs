mod db;
mod diff_router;
mod github;
mod github_router;
mod logger;
mod models;
mod utils;

use db::MongoDbFairing;
use diff_router::AppConfig;
use dotenv::dotenv;
use github::GithubClient;
use rocket_cors::{AllowedOrigins, Cors, CorsOptions};
use std::env;
use std::sync::Arc;

#[macro_use]
extern crate rocket;


fn instantiate_cors(allowed: &str) -> Cors {
    CorsOptions {
        allowed_origins: AllowedOrigins::some_exact(&[allowed]),
        ..CorsOptions::default()
    }
    .to_cors()
    .expect("Failed to instantiate CORS")
}

#[launch]
async fn rocket() -> _ {
    info!("hello");

    let _log = logger::setup_logging().expect("Can't instantiate logger");

    dotenv().ok();

    let allowed_origin_str = env::var("ORIGINS").expect("Invalid ORIGINS");
    let access_token = env::var("ACCESS_TOKEN_GH").expect("No Github token");
    let organization_name = env::var("ORGANIZATION_GH").expect("No Github organization name");
    let hostname_gh = env::var("HOSTNAME_GH").expect("No Github hostname");
    let folder_a = env::var("FOLDER_A_NAME").expect("No folder name");
    let folder_b = env::var("FOLDER_B_NAME").expect("No folder name");

    let app_config = AppConfig { folder_a, folder_b };

    info!("Allowed origins are {}", allowed_origin_str);

    let github_client = GithubClient::new(access_token, organization_name, hostname_gh)
        .await
        .expect("Failed to create GithubClient");

    rocket::build()
        .attach(instantiate_cors(&allowed_origin_str))
        .attach(MongoDbFairing)
        .manage(Arc::new(github_client))
        .manage(Arc::new(app_config))
        .mount(
            "/",
            routes![
                github_router::simple_json,
                github_router::get_repo_contents,
                github_router::get_repo,
                github_router::get_nb_repo,
                github_router::get_list_of_repos,
                github_router::get_config_from_stack_and_file,
                github_router::get_config_from_stack_and_file_string,
                github_router::get_repo_all_contents,
                diff_router::get_diff_by_id,
                diff_router::get_configs_from_stacks_name,
                diff_router::insert_diff,
                diff_router::get_all_diffs_from_stacks,
                diff_router::get_latest_diffs_from_stacks,
                diff_router::toggle_review_endpoint,
                diff_router::compute_diff_for_all_files,
            ],
        )
}
