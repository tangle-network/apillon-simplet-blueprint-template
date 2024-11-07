use super::{
    deploy_service, ApillonSimpletsDocker, CommonConfig, ServiceConfig, ServiceType,
    SimpletsBuilder, SmtpConfig,
};
use gadget_sdk::docker::bollard;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmailAirdropConfig {
    common: CommonConfig,
    collection_uuid: Option<String>,
}

impl ServiceConfig for EmailAirdropConfig {
    fn into_env_vars(self) -> HashMap<String, String> {
        let mut env_vars = self.common.build_env_vars();
        if let Some(uuid) = self.collection_uuid {
            env_vars.insert("COLLECTION_UUID".to_string(), uuid);
        }
        env_vars
    }
}

pub struct EmailAirdropBuilder {
    config: EmailAirdropConfig,
}

#[async_trait::async_trait]
impl SimpletsBuilder for EmailAirdropBuilder {
    type Config = EmailAirdropConfig;

    fn new() -> Self {
        Self {
            config: EmailAirdropConfig {
                common: CommonConfig {
                    app_secret: None,
                    app_url: None,
                    mysql_password: None,
                    mysql_db: None,
                    admin_wallet: None,
                    apillon_key: None,
                    apillon_secret: None,
                    smtp_config: None,
                },
                collection_uuid: None,
            },
        }
    }

    fn get_config(&self) -> &Self::Config {
        &self.config
    }

    fn get_config_mut(&mut self) -> &mut Self::Config {
        &mut self.config
    }

    fn app_secret(mut self, secret: impl Into<String>) -> Self {
        self.config.common.app_secret = Some(secret.into());
        self
    }

    fn app_url(mut self, url: impl Into<String>) -> Self {
        self.config.common.app_url = Some(url.into());
        self
    }

    fn mysql_password(mut self, password: impl Into<String>) -> Self {
        self.config.common.mysql_password = Some(password.into());
        self
    }

    fn mysql_db(mut self, db: impl Into<String>) -> Self {
        self.config.common.mysql_db = Some(db.into());
        self
    }

    fn admin_wallet(mut self, wallet: impl Into<String>) -> Self {
        self.config.common.admin_wallet = Some(wallet.into());
        self
    }

    fn apillon_credentials(mut self, key: impl Into<String>, secret: impl Into<String>) -> Self {
        self.config.common.apillon_key = Some(key.into());
        self.config.common.apillon_secret = Some(secret.into());
        self
    }

    fn smtp_config(mut self, smtp_config: SmtpConfig) -> Self {
        self.config.common.smtp_config = Some(smtp_config);
        self
    }

    async fn deploy(self) -> Result<ApillonSimpletsDocker, bollard::errors::Error> {
        deploy_service(self.config, ServiceType::EmailAirdrop).await
    }
}

// Add service-specific method
impl EmailAirdropBuilder {
    pub fn collection_uuid(mut self, uuid: impl Into<String>) -> Self {
        self.config.collection_uuid = Some(uuid.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_email_airdrop_deployment() {
        let airdrop = EmailAirdropBuilder::new()
            .app_secret("test_secret")
            .app_url("http://localhost:8080")
            .mysql_password("test_password")
            .mysql_db("airdrop_test")
            .admin_wallet("0x123...")
            .apillon_credentials("test_key", "test_secret")
            .collection_uuid("test-uuid")
            .smtp_config(SmtpConfig {
                host: "smtp.test.com".to_string(),
                port: "587".to_string(),
                username: "test".to_string(),
                password: "test".to_string(),
                email_from: "test@test.com".to_string(),
                name_from: "Test Sender".to_string(),
            })
            .deploy()
            .await;

        assert!(airdrop.is_ok());
    }
}
