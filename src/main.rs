use std::collections::HashMap;
use std::sync::Arc;

use apillon_simplet_blueprint_template as blueprint;
use color_eyre::Result;
use gadget_sdk as sdk;
use gadget_sdk::runners::tangle::TangleConfig;
use gadget_sdk::runners::BlueprintRunner;
use sdk::tangle_subxt::*;
use tokio::sync::RwLock;

#[sdk::main(env)]
async fn main() -> Result<()> {
    let signer = env.first_sr25519_signer()?;
    let client = subxt::OnlineClient::from_url(&env.ws_rpc_endpoint).await?;

    let service_id = env.service_id().expect("should exist");

    let context = blueprint::SimpletsContext {
        config: env.clone(),
        simplet_configs: HashMap::new(),
        running_services: Arc::new(RwLock::new(HashMap::new())),
    };

    // Create the event handler from the job
    let run_poa_simplet = blueprint::RunProofOfAttendanceSimpletEventHandler {
        service_id,
        client: client.clone(),
        signer: signer.clone(),
        context: context.clone(),
    };

    let run_email_airdrop = blueprint::RunEmailAirdropSimpletEventHandler {
        service_id,
        client: client.clone(),
        signer: signer.clone(),
        context: context.clone(),
    };

    tracing::info!("Starting the event watcher ...");
    BlueprintRunner::new(TangleConfig::default(), env)
        .job(run_poa_simplet)
        .job(run_email_airdrop)
        .run()
        .await?;

    tracing::info!("Exiting...");
    Ok(())
}
