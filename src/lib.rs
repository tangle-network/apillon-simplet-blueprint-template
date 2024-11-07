use gadget_sdk::tangle_subxt::tangle_testnet_runtime::api;
use gadget_sdk::{self as sdk, error};
use simplets::email_airdrop::EmailAirdropBuilder;
use std::sync::Arc;
use std::{collections::HashMap, convert::Infallible};
use tokio::sync::RwLock;

use api::services::events::JobCalled;
use sdk::event_listener::tangle::{jobs::services_pre_processor, TangleEventListener};

pub mod simplets;
use simplets::proof_of_attendance::ProofOfAttendanceBuilder;
use simplets::{CommonConfig, SimpletsBuilder};

#[derive(Clone)]
pub struct SimpletsContext {
    pub simplet_configs: HashMap<String, CommonConfig>,
    pub config: sdk::config::StdGadgetConfiguration,
    pub running_services: Arc<RwLock<HashMap<String, simplets::ApillonSimpletsDocker>>>,
}

#[sdk::job(
    id = 0,
    params(custom_config),
    result(_),
    event_listener(
        listener = TangleEventListener::<SimpletsContext, JobCalled>,
        pre_processor = services_pre_processor,
    ),
)]
pub async fn run_proof_of_attendance_simplet(
    custom_config: Vec<u8>,
    context: SimpletsContext,
) -> Result<String, Infallible> {
    // Extract configuration values from context
    let config = context.simplet_configs.get("proof_of_attendance").unwrap();
    let custom_config = serde_json::from_slice::<CommonConfig>(&custom_config[..]).unwrap();

    // Build and deploy the Proof of Attendance simplet
    let mut builder = ProofOfAttendanceBuilder::new();

    if let Some(secret) = config
        .app_secret
        .as_ref()
        .or(custom_config.app_secret.as_ref())
    {
        builder = builder.app_secret(secret);
    }
    if let Some(url) = config.app_url.as_ref().or(custom_config.app_url.as_ref()) {
        builder = builder.app_url(url);
    }
    if let Some(password) = config
        .mysql_password
        .as_ref()
        .or(custom_config.mysql_password.as_ref())
    {
        builder = builder.mysql_password(password);
    }
    if let Some(db) = config.mysql_db.as_ref().or(custom_config.mysql_db.as_ref()) {
        builder = builder.mysql_db(db);
    }
    if let Some(wallet) = config
        .admin_wallet
        .as_ref()
        .or(custom_config.admin_wallet.as_ref())
    {
        builder = builder.admin_wallet(wallet);
    }
    if let (Some(key), Some(secret)) = (
        config
            .apillon_key
            .as_ref()
            .or(custom_config.apillon_key.as_ref()),
        config
            .apillon_secret
            .as_ref()
            .or(custom_config.apillon_secret.as_ref()),
    ) {
        builder = builder.apillon_credentials(key, secret);
    }
    if let Some(smtp) = config
        .smtp_config
        .as_ref()
        .or(custom_config.smtp_config.as_ref())
    {
        builder = builder.smtp_config(simplets::SmtpConfig {
            host: smtp.host.clone(),
            port: smtp.port.clone(),
            username: smtp.username.clone(),
            password: smtp.password.clone(),
            email_from: smtp.email_from.clone(),
            name_from: smtp.name_from.clone(),
        });
    }

    let unique_id = builder.get_unique_id();
    match builder.deploy().await {
        Ok(poa) => {
            // Store the running service in the context
            context
                .running_services
                .write()
                .await
                .insert(format!("proof_of_attendance_{}", unique_id), poa);
            Ok("Proof of Attendance simplet deployed successfully!".to_string())
        }
        Err(e) => {
            // Since we're returning Result<String, Infallible>, we need to handle any error
            // by panicking since Infallible means this function cannot fail
            error!("Failed to deploy Proof of Attendance simplet: {:?}", e);
            Ok("Failed to deploy Proof of Attendance simplet!".to_string())
        }
    }
}

#[sdk::job(
    id = 1,
    params(custom_config),
    result(_),
    event_listener(
        listener = TangleEventListener::<SimpletsContext, JobCalled>,
        pre_processor = services_pre_processor,
    ),
)]
pub async fn run_email_airdrop_simplet(
    custom_config: Vec<u8>,
    context: SimpletsContext,
) -> Result<String, Infallible> {
    let config = context.simplet_configs.get("email_airdrop").unwrap();
    let custom_config = serde_json::from_slice::<CommonConfig>(&custom_config[..]).unwrap();

    // Build and deploy the Email Airdrop simplet
    let mut builder = EmailAirdropBuilder::new();

    if let Some(secret) = config
        .app_secret
        .as_ref()
        .or(custom_config.app_secret.as_ref())
    {
        builder = builder.app_secret(secret);
    }
    if let Some(url) = config.app_url.as_ref().or(custom_config.app_url.as_ref()) {
        builder = builder.app_url(url);
    }
    if let Some(password) = config
        .mysql_password
        .as_ref()
        .or(custom_config.mysql_password.as_ref())
    {
        builder = builder.mysql_password(password);
    }
    if let Some(db) = config.mysql_db.as_ref().or(custom_config.mysql_db.as_ref()) {
        builder = builder.mysql_db(db);
    }
    if let Some(wallet) = config
        .admin_wallet
        .as_ref()
        .or(custom_config.admin_wallet.as_ref())
    {
        builder = builder.admin_wallet(wallet);
    }
    if let (Some(key), Some(secret)) = (
        config
            .apillon_key
            .as_ref()
            .or(custom_config.apillon_key.as_ref()),
        config
            .apillon_secret
            .as_ref()
            .or(custom_config.apillon_secret.as_ref()),
    ) {
        builder = builder.apillon_credentials(key, secret);
    }
    if let Some(smtp) = config
        .smtp_config
        .as_ref()
        .or(custom_config.smtp_config.as_ref())
    {
        builder = builder.smtp_config(simplets::SmtpConfig {
            host: smtp.host.clone(),
            port: smtp.port.clone(),
            username: smtp.username.clone(),
            password: smtp.password.clone(),
            email_from: smtp.email_from.clone(),
            name_from: smtp.name_from.clone(),
        });
    }

    let unique_id = builder.get_unique_id();
    match builder.deploy().await {
        Ok(airdrop) => {
            // Store the running service in the context
            context
                .running_services
                .write()
                .await
                .insert(format!("email_airdrop_{}", unique_id), airdrop);
            Ok("Email Airdrop simplet deployed successfully!".to_string())
        }
        Err(e) => {
            // Since we're returning Result<String, Infallible>, we need to handle any error
            // by panicking since Infallible means this function cannot fail
            error!("Failed to deploy Email Airdrop simplet: {:?}", e);
            Ok("Failed to deploy Email Airdrop simplet!".to_string())
        }
    }
}
