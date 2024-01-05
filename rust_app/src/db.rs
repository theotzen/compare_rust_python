use bson::doc;
use chrono::Utc;
use mongodb;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::futures::TryStreamExt;
use rocket::{Build, Rocket};
use std::env;
use std::error::Error;
use std::time::SystemTime;

use crate::{models, rocket};

pub struct DiffCollection {
    pub collection: mongodb::Collection<models::FileDiff>,
}

impl DiffCollection {
    pub async fn init(database: &mongodb::Database) -> Self {
        let collection = database.collection::<models::FileDiff>("diffs");
        DiffCollection { collection }
    }

    pub async fn find_diff_by_id(
        &self,
        diff_id: bson::oid::ObjectId,
    ) -> Result<models::FileDiff, Box<dyn Error>> {
        match self.collection.find_one(doc! {"_id": diff_id}, None).await {
            Ok(Some(document)) => Ok(document),
            Ok(None) => Err(anyhow::Error::msg("Not Found").into()),
            Err(e) => Err(Box::new(e) as Box<dyn Error>),
        }
    }

    pub async fn insert_diff_from_diff_base_schema(
        &self,
        payload: models::DiffBaseSchema,
    ) -> Result<mongodb::results::InsertOneResult, Box<dyn Error>> {
        let now = Utc::now();
        let system_time: SystemTime = now.into();
        let file_diff = models::FileDiff {
            id: None,
            stack_a: payload.stack_a,
            stack_b: payload.stack_b,
            file: payload.file,
            left_not_right: payload.left_not_right,
            right_not_left: payload.right_not_left,
            same_key_diff_value: payload.same_key_diff_value,
            reviewed: Some("false".to_string()),
            created_at: Some(system_time.into()),
            updated_at: Some(system_time.into()),
        };
        self.collection
            .insert_one(file_diff, None)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error>)
    }

    pub async fn insert_diff(
      &self,
      payload: &models::FileDiff,
  ) -> Result<mongodb::results::InsertOneResult, Box<dyn Error>> {
      info!("Inserting for file : {}", &payload.file);
      self.collection
          .insert_one(payload, None)
          .await
          .map_err(|e| Box::new(e) as Box<dyn Error>)
  }

    pub async fn get_all_diffs_from_stacks(
        &self,
        payload: &models::GetAllDiffsPayload,
    ) -> Result<Vec<models::FileDiff>, Box<dyn Error>> {
        match self
            .collection
            .find(
                doc! {
                  "stack_a": &payload.stack_a,
                  "stack_b": &payload.stack_b,
                },
                None,
            )
            .await
        {
            Ok(cursor) => cursor
                .try_collect()
                .await
                .map_err(|e| Box::new(e) as Box<dyn Error>),
            Err(e) => Err(Box::new(e) as Box<dyn Error>),
        }
    }

    pub async fn toggle_review(
        &self,
        diff_id: bson::oid::ObjectId,
    ) -> Result<mongodb::results::UpdateResult, Box<dyn Error>> {
        let file_diff = self.find_diff_by_id(diff_id).await?;
        info!("{:?}", &file_diff.reviewed);
        let reviewed = match file_diff.reviewed {
            Some(ref r) if r == "false" => "true".to_string(),
            Some(ref r) if r == "true" => "false".to_string(),
            _ => return Err(anyhow::Error::msg("Not Found").into()),
        };
        info!("{:?}", &reviewed);

        let new_doc = doc! {
          "$set":
          {
            "reviewed": reviewed
          },
        };

        self.collection
            .update_one(doc! {"_id": diff_id}, new_doc, None)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error>)
    }
}
pub struct MongoDb {
    pub client: mongodb::Client,
    pub database: mongodb::Database,
}

impl MongoDb {
    pub async fn init() -> Self {
        let database_url = env::var("DATABASE_URL").expect("Can't use DATABASE_URL");

        let client_options = mongodb::options::ClientOptions::parse(&database_url)
            .await
            .expect("Failed to parse client options");

        let client = mongodb::Client::with_options(client_options)
            .expect("Failed to initialize MongoDB client");

        // Ping the server to check if the connection is actually established
        match client
            .database("admin")
            .run_command(doc! {"ping": 1}, None)
            .await
        {
            Ok(_) => {
                let database_name =
                    env::var("MONGO_INITDB_DATABASE").expect("MONGO_INITDB_DATABASE must be set");
                let database = client.database(&database_name);

                info!(
                    "Successfully connected to MongoDB at database {}",
                    &database.name()
                );
                MongoDb { client, database }
            }
            Err(e) => {
                error!("Failed to connect to MongoDB: {}", e);
                panic!("MongoDB connection failed: {}", e);
            }
        }
    }
}

pub struct MongoDbFairing;
#[rocket::async_trait]
impl Fairing for MongoDbFairing {
    fn info(&self) -> Info {
        Info {
            name: "MongoDB Connection Fairing",
            kind: Kind::Ignite | Kind::Liftoff,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> rocket::fairing::Result {
        let mongodb = MongoDb::init().await;
        let mongodb_collection = DiffCollection::init(&mongodb.database).await;
        Ok(rocket.manage(mongodb_collection))
    }
}
