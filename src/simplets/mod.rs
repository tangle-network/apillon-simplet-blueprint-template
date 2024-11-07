use gadget_sdk::docker::{bollard, connect_to_docker, Container};
use gadget_sdk::subxt_core::ext::sp_core::bytes::to_hex;
use gadget_sdk::subxt_core::ext::sp_core::keccak_256;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

pub mod email_airdrop;
pub mod proof_of_attendance;

#[async_trait::async_trait]
pub trait SimpletsBuilder {
    type Config: Clone + Serialize;

    fn new() -> Self;
    fn app_secret(self, secret: impl Into<String>) -> Self;
    fn app_url(self, url: impl Into<String>) -> Self;
    fn mysql_password(self, password: impl Into<String>) -> Self;
    fn mysql_db(self, db: impl Into<String>) -> Self;
    fn admin_wallet(self, wallet: impl Into<String>) -> Self;
    fn apillon_credentials(self, key: impl Into<String>, secret: impl Into<String>) -> Self;
    fn smtp_config(self, smtp_config: SmtpConfig) -> Self;

    fn get_config(&self) -> &Self::Config;
    fn get_config_mut(&mut self) -> &mut Self::Config;
    fn get_unique_id(&self) -> String {
        // Create a unique hash based on config values
        let config = self.get_config();
        let serialized = serde_json::to_string(config).unwrap();
        let hash = keccak_256(serialized.as_bytes());
        to_hex(&hash[..], false)
    }

    async fn deploy(self) -> Result<ApillonSimpletsDocker, bollard::errors::Error>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub email_from: String,
    pub name_from: String,
}

pub trait ServiceConfig {
    fn into_env_vars(self) -> HashMap<String, String>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommonConfig {
    pub app_secret: Option<String>,
    pub app_url: Option<String>,
    pub mysql_password: Option<String>,
    pub mysql_db: Option<String>,
    pub admin_wallet: Option<String>,
    pub apillon_key: Option<String>,
    pub apillon_secret: Option<String>,
    pub smtp_config: Option<SmtpConfig>,
}

impl CommonConfig {
    pub fn build_env_vars(&self) -> HashMap<String, String> {
        let mut env_vars = HashMap::new();

        // Add optional configurations
        if let Some(secret) = &self.app_secret {
            env_vars.insert("APP_SECRET".to_string(), secret.clone());
        }
        if let Some(url) = &self.app_url {
            env_vars.insert("APP_URL".to_string(), url.clone());
        }
        if let Some(password) = &self.mysql_password {
            env_vars.insert("MYSQL_PASSWORD".to_string(), password.clone());
        }
        if let Some(db) = &self.mysql_db {
            env_vars.insert("MYSQL_DB".to_string(), db.clone());
        }
        if let Some(wallet) = &self.admin_wallet {
            env_vars.insert("ADMIN_WALLET".to_string(), wallet.clone());
        }
        if let Some(key) = &self.apillon_key {
            env_vars.insert("APILLON_KEY".to_string(), key.clone());
        }
        if let Some(secret) = &self.apillon_secret {
            env_vars.insert("APILLON_SECRET".to_string(), secret.clone());
        }

        // Add SMTP configuration if present
        if let Some(smtp) = &self.smtp_config {
            env_vars.insert("SMTP_HOST".to_string(), smtp.host.clone());
            env_vars.insert("SMTP_PORT".to_string(), smtp.port.clone());
            env_vars.insert("SMTP_USERNAME".to_string(), smtp.username.clone());
            env_vars.insert("SMTP_PASSWORD".to_string(), smtp.password.clone());
            env_vars.insert("SMTP_EMAIL_FROM".to_string(), smtp.email_from.clone());
            env_vars.insert("SMTP_NAME_FROM".to_string(), smtp.name_from.clone());
        }

        env_vars
    }
}

#[derive(Clone)]
pub struct ApillonSimpletsDocker {
    docker: Arc<bollard::Docker>,
    env_vars: HashMap<String, String>,
    service_type: ServiceType,
}

#[derive(Clone)]
pub enum ServiceType {
    ProofOfAttendance,
    EmailAirdrop,
}

impl ServiceType {
    fn get_db_name(&self) -> &'static str {
        match self {
            ServiceType::ProofOfAttendance => "poa_db",
            ServiceType::EmailAirdrop => "airdrop_db",
        }
    }

    fn get_app_image(&self) -> &'static str {
        match self {
            ServiceType::ProofOfAttendance => "ps-poa:latest",
            ServiceType::EmailAirdrop => "ps-email-airdrop:latest",
        }
    }
}

impl ApillonSimpletsDocker {
    pub fn new(
        docker: Arc<bollard::Docker>,
        env_vars: HashMap<String, String>,
        service_type: ServiceType,
    ) -> Self {
        Self {
            docker,
            env_vars,
            service_type,
        }
    }

    pub async fn start(&self) -> Result<(), bollard::errors::Error> {
        // Start MySQL container first
        let db_env = vec![
            format!(
                "MYSQL_ROOT_PASSWORD={}",
                self.env_vars
                    .get("MYSQL_PASSWORD")
                    .unwrap_or(&"root".to_string())
            ),
            format!(
                "MYSQL_DATABASE={}",
                self.env_vars.get("MYSQL_DB").unwrap_or(&"poa".to_string())
            ),
        ];

        let mut db_container = Container::new(&self.docker, "mysql");
        db_container
            .env(db_env)
            .binds(["./mysql-data:/var/lib/mysql"]);

        db_container.create().await?;
        db_container.start(false).await?;

        // Wait for MySQL to be healthy
        self.wait_for_mysql(&db_container).await?;

        // Start app container with updated configuration
        let app_env = self.build_app_environment();

        let mut app_container = Container::new(&self.docker, self.service_type.get_app_image());
        app_container.env(app_env).binds(["./app-data:/app/data"]);

        app_container.create().await?;
        app_container.start(false).await?;

        Ok(())
    }

    async fn wait_for_mysql(
        &self,
        container: &Container<'_>,
    ) -> Result<(), bollard::errors::Error> {
        let max_attempts = 30;
        let mut attempts = 0;

        while attempts < max_attempts {
            tokio::time::sleep(Duration::from_secs(2)).await;

            if let Some(id) = container.id() {
                if let Ok(info) = self.docker.inspect_container(id, None).await {
                    if let Some(state) = info.state {
                        if state.status == Some(bollard::secret::ContainerStateStatusEnum::RUNNING)
                        {
                            return Ok(());
                        }
                    }
                }
            }

            attempts += 1;
        }

        Err(bollard::errors::Error::IOError {
            err: std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "MySQL failed to become healthy",
            ),
        })
    }

    fn build_app_environment(&self) -> Vec<String> {
        let mut app_env = vec![
            "APP_ENV=production".to_string(),
            format!(
                "APP_SECRET={}",
                self.env_vars
                    .get("APP_SECRET")
                    .unwrap_or(&"secret".to_string())
            ),
            format!(
                "APP_URL={}",
                self.env_vars
                    .get("APP_URL")
                    .unwrap_or(&"http://localhost:3000".to_string())
            ),
            "API_PORT=3000".to_string(),
            "API_HOST=0.0.0.0".to_string(),
            format!("MYSQL_HOST={}", self.service_type.get_db_name()),
            "MYSQL_PORT=3306".to_string(),
            format!(
                "MYSQL_DB={}",
                self.env_vars.get("MYSQL_DB").unwrap_or(&"poa".to_string())
            ),
            "MYSQL_USER=root".to_string(),
            format!(
                "MYSQL_PASSWORD={}",
                self.env_vars
                    .get("MYSQL_PASSWORD")
                    .unwrap_or(&"root".to_string())
            ),
            "MYSQL_POOL=5".to_string(),
        ];

        // Add optional environment variables
        let optional_vars = [
            "ADMIN_WALLET",
            "APILLON_KEY",
            "APILLON_SECRET",
            "COLLECTION_UUID",
            "SMTP_HOST",
            "SMTP_PORT",
            "SMTP_USERNAME",
            "SMTP_PASSWORD",
            "SMTP_EMAIL_FROM",
            "SMTP_NAME_FROM",
        ];

        for var in optional_vars.iter() {
            if let Some(value) = self.env_vars.get(*var) {
                app_env.push(format!("{}={}", var, value));
            }
        }

        app_env
    }

    pub async fn stop(&self) -> Result<(), bollard::errors::Error> {
        let mut app_container = Container::new(&self.docker, self.service_type.get_app_image());
        let mut db_container = Container::new(&self.docker, "mysql");

        app_container.stop().await?;
        db_container.stop().await?;
        Ok(())
    }

    pub async fn cleanup(self) -> Result<(), bollard::errors::Error> {
        let force_options = bollard::container::RemoveContainerOptions {
            force: true,
            ..Default::default()
        };

        let app_container = Container::new(&self.docker, self.service_type.get_app_image());
        let db_container = Container::new(&self.docker, "mysql");

        app_container.remove(Some(force_options)).await?;
        db_container.remove(Some(force_options)).await?;
        Ok(())
    }
}

pub async fn deploy_service<T: ServiceConfig>(
    config: T,
    service_type: ServiceType,
) -> Result<ApillonSimpletsDocker, bollard::errors::Error> {
    let env_vars = config.into_env_vars();
    let docker = connect_to_docker(None).await?;
    let simplets = ApillonSimpletsDocker::new(docker, env_vars, service_type);
    simplets.start().await?;
    Ok(simplets)
}
