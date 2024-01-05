use base64::decode;
use log::info;
use octocrab::{Octocrab, OctocrabBuilder};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize)]
pub struct SerializableContentItems {
    pub items: Vec<SerializableContent>,
}

#[derive(Serialize, Deserialize)]
pub struct SerializableContent {
    pub name: String,
    pub path: String,
    pub content: Option<String>,
    pub url: String,
    pub html_url: Option<String>,
    pub git_url: Option<String>,
    pub download_url: Option<String>,
    pub r#type: String,
}

#[derive(Debug)]
pub enum ConfigError {
    NotFound(String),
    NoContent,
    DecodeError(base64::DecodeError),
    Utf8Error(std::string::FromUtf8Error),
    OctocrabError(octocrab::Error),
    Other(anyhow::Error),
}

impl From<octocrab::Error> for ConfigError {
    fn from(err: octocrab::Error) -> Self {
        ConfigError::OctocrabError(err)
    }
}

impl From<base64::DecodeError> for ConfigError {
    fn from(err: base64::DecodeError) -> Self {
        ConfigError::DecodeError(err)
    }
}

impl From<std::string::FromUtf8Error> for ConfigError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        ConfigError::Utf8Error(err)
    }
}

impl From<anyhow::Error> for ConfigError {
    fn from(err: anyhow::Error) -> Self {
        ConfigError::Other(err)
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::NotFound(file) => write!(f, "File not found: {}", file),
            ConfigError::NoContent => write!(f, "No content available"),
            ConfigError::DecodeError(err) => write!(f, "Decode error: {}", err),
            ConfigError::Utf8Error(err) => write!(f, "UTF-8 conversion error: {}", err),
            ConfigError::OctocrabError(err) => write!(f, "Octocrab error {}", err),
            ConfigError::Other(err) => write!(f, "Other error: {}", err),
        }
    }
}
impl std::error::Error for ConfigError {}

#[derive(Clone)]
pub struct GithubClient {
    octocrab: Octocrab,
    organization_name: String,
}

impl From<octocrab::models::repos::ContentItems> for SerializableContentItems {
    fn from(content_items: octocrab::models::repos::ContentItems) -> Self {
        SerializableContentItems {
            items: content_items.items.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<octocrab::models::repos::Content> for SerializableContent {
    fn from(content: octocrab::models::repos::Content) -> Self {
        SerializableContent {
            name: content.name,
            path: content.path,
            content: content.content,
            url: content.url,
            html_url: content.html_url,
            git_url: content.git_url,
            download_url: content.download_url,
            r#type: content.r#type,
        }
    }
}

impl GithubClient {
    pub async fn new(
        access_token: String,
        organization_name: String,
        hostname_gh: String,
    ) -> Result<Self, anyhow::Error> {
        let octocrab = OctocrabBuilder::new()
            .base_uri(format!("https://{}/api/v3", hostname_gh))?
            .personal_token(access_token)
            .build()?;

        match octocrab.current().user().await {
            Ok(user) => {
                info!(
                    "Authenticated as {}, {} & {}",
                    user.login, user.r#type, user.id
                )
            }
            Err(e) => {
                info!("Authentication failed. Please check your access token");
                return Err(e.into());
            }
        }

        Ok(GithubClient {
            octocrab,
            organization_name,
        })
    }

    pub async fn get_number_of_repo(&self) -> Result<usize, anyhow::Error> {
        let repos = self
            .octocrab
            .current()
            .list_repos_for_authenticated_user()
            .page(1)
            .send()
            .await;

        match repos {
            Ok(repo) => Ok(repo.items.len()),
            Err(e) => {
                return Err(e.into());
            }
        }
    }

    pub async fn get_list_of_repos(
        &self,
    ) -> Result<Vec<octocrab::models::Repository>, anyhow::Error> {
        let repos = self
            .octocrab
            .current()
            .list_repos_for_authenticated_user()
            .per_page(100)
            .send()
            .await;

        match repos {
            Ok(page) => {
                if page.next.is_some() {
                    info!("There are some more pages to fetch !")
                }
                Ok(page.items)
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }

    pub async fn get_repo(
        &self,
        repository_name: &str,
    ) -> Result<octocrab::models::Repository, anyhow::Error> {
        info!(
            "Getting repository {} for organization {}",
            self.organization_name, repository_name
        );

        let result = self
            .octocrab
            .repos(&self.organization_name, repository_name)
            .get()
            .await;

        match result {
            Ok(repo) => {
                info!(
                    "Successfully retrieved info from {}, {}",
                    repository_name, repo.id
                );
                Ok(repo)
            }
            Err(e) => {
                error!(
                    "Cannot retrieve info for {}, because {}",
                    repository_name, e
                );
                Err(e.into())
            }
        }
    }

    pub async fn get_contents(
        &self,
        repository_name: &str,
        folder_path: &str,
    ) -> Result<SerializableContentItems, anyhow::Error> {
        let content_items = self
            .octocrab
            .repos(&self.organization_name, repository_name)
            .get_content()
            .path(folder_path)
            .send()
            .await;

        match content_items {
            Ok(repo) => {
                info!("Successfully retrieved info from {}", repository_name);
                Ok(repo.into())
            }
            Err(e) => {
                error!(
                    "Cannot retrieve info for {}, because {}",
                    repository_name, e
                );
                Err(e.into())
            }
        }
    }

    pub async fn get_contents_for_repo(
        &self,
        repository_name: &str,
        path: &str,
    ) -> Result<SerializableContentItems, anyhow::Error> {
        let content_items = self
            .octocrab
            .repos(&self.organization_name, repository_name)
            .get_content()
            .path(format!("/{}", path))
            .send()
            .await;

        match content_items {
            Ok(repo) => {
                info!("Successfully retrieved info from {}", repository_name);
                Ok(repo.into())
            }
            Err(e) => {
                error!(
                    "Cannot retrieve info for {}, because {}",
                    repository_name, e
                );
                Err(e.into())
            }
        }
    }

    pub async fn get_config_from_stack_and_file(
        &self,
        stack_a: &str,
        file: &str,
    ) -> Result<SerializableContentItems, anyhow::Error> {
        let content = self
            .octocrab
            .repos(&self.organization_name, stack_a)
            .get_content()
            .path(file)
            .send()
            .await;

        match content {
            Ok(cont) => Ok(cont.into()),
            Err(e) => {
                error!("Cannot retrieve content ; {}", e);
                Err(e.into())
            }
        }
    }

    pub async fn _get_config_from_stack_and_file_string(
        &self,
        stack_a: &str,
        file: &str,
    ) -> Result<String, anyhow::Error> {
        let content = self
            .octocrab
            .repos(&self.organization_name, stack_a)
            .get_content()
            .path(file)
            .send()
            .await?;

        if let Some(content_item) = content.items.into_iter().next() {
            if let Some(content_string) = content_item.content {
                // Create a new String without newlines and carriage returns
                let cleaned_encoded_string = content_string.replace("\n", "").replace("\r", "");
                // Trim the string to remove any leading or trailing whitespace
                let trimmed_encoded_string = cleaned_encoded_string.trim();
                // Now you can pass a reference to `trimmed_encoded_string` to `decode`
                let decoded = decode(&trimmed_encoded_string);

                match decoded {
                    Ok(dec) => {
                        let reencoded = String::from_utf8(dec);
                        match reencoded {
                            Ok(reenco) => Ok(reenco),
                            Err(e) => {
                                error!("Cannot reencode ; {}", e);
                                Err(e.into())
                            }
                        }
                    }
                    Err(e) => {
                        error!("Cannot decode ; {}", e);
                        Err(e.into())
                    }
                }
            } else {
                Err(anyhow::Error::msg("No content available"))
            }
        } else {
            Err(anyhow::Error::msg("No item in page"))
        }
    }

    pub async fn get_config_from_stack_and_file_string(
        &self,
        stack: &str,
        file: &str,
    ) -> Result<String, ConfigError> {
        let content = self
            .octocrab
            .repos(&self.organization_name, stack)
            .get_content()
            .path(file)
            .send()
            .await
            .map_err(ConfigError::from)?;

        let content_item = content
            .items
            .into_iter()
            .next()
            .ok_or(ConfigError::NotFound(file.to_string()))?;


        let content_string = content_item.content.ok_or(ConfigError::NoContent)?;

        let cleaned_encoded_string = content_string.replace("\n", "").replace("\r", "");
        let trimed_encoded_string = cleaned_encoded_string.trim();

        let decoded_string = decode(trimed_encoded_string).map_err(ConfigError::from)?;

        String::from_utf8(decoded_string).map_err(ConfigError::from)
    }
}
