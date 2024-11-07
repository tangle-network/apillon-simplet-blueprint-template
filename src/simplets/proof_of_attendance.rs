use super::{
    deploy_service, ApillonSimpletsDocker, CommonConfig, ServiceConfig, ServiceType,
    SimpletsBuilder, SmtpConfig,
};
use gadget_sdk::docker::bollard;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofOfAttendanceConfig {
    common: CommonConfig,
}

impl ServiceConfig for ProofOfAttendanceConfig {
    fn into_env_vars(self) -> HashMap<String, String> {
        self.common.build_env_vars()
    }
}

pub struct ProofOfAttendanceBuilder {
    config: ProofOfAttendanceConfig,
}

#[async_trait::async_trait]
impl SimpletsBuilder for ProofOfAttendanceBuilder {
    type Config = ProofOfAttendanceConfig;

    fn new() -> Self {
        Self {
            config: ProofOfAttendanceConfig {
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
        deploy_service(self.config, ServiceType::ProofOfAttendance).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_proof_of_attendance_deployment() {
        let poa = ProofOfAttendanceBuilder::new()
            .app_secret("test_secret")
            .app_url("http://localhost:8080")
            .mysql_password("test_password")
            .mysql_db("poa_test")
            .admin_wallet("0x123...")
            .apillon_credentials("test_key", "test_secret")
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

        assert!(poa.is_ok());
    }
}
