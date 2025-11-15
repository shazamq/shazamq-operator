// Copyright (c) 2025 Murtaza Shajapurwala
//
// Shazamq Operator - Kubernetes Operator for Shazamq Clusters

use anyhow::Result;
use kube::{
    runtime::{controller::Action, Controller},
    Api, Client, ResourceExt,
};
use std::sync::Arc;
use tokio::time::Duration;
use tracing::{error, info, warn};

mod crd;
mod reconciler;

use crd::ShazamqCluster;
use reconciler::Reconciler;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .json()
        .init();

    info!("╔═══════════════════════════════════════════════════════╗");
    info!("║            Shazamq Kubernetes Operator                ║");
    info!("║                  Version 0.1.0                         ║");
    info!("╚═══════════════════════════════════════════════════════╝");

    // Create Kubernetes client
    let client = Client::try_default().await?;
    info!("Connected to Kubernetes cluster");

    // Create API for ShazamqCluster resources
    let api: Api<ShazamqCluster> = Api::all(client.clone());
    
    // Create reconciler
    let reconciler = Arc::new(Reconciler::new(client.clone()));
    
    info!("Starting controller...");
    
    // Start the controller
    Controller::new(api, Default::default())
        .run(
            move |obj, ctx| {
                let reconciler = ctx.clone();
                async move { reconciler.reconcile(obj).await }
            },
            |obj, error, _ctx| {
                error!(
                    name = obj.name_any(),
                    namespace = obj.namespace(),
                    error = %error,
                    "Reconciliation error"
                );
                Action::requeue(Duration::from_secs(60))
            },
            reconciler,
        )
        .for_each(|res| async move {
            match res {
                Ok((obj, action)) => {
                    info!(
                        name = obj.name_any(),
                        namespace = obj.namespace(),
                        requeue_after = ?action.requeue_after,
                        "Reconciled"
                    );
                }
                Err(e) => {
                    error!(error = %e, "Controller error");
                }
            }
        })
        .await;

    Ok(())
}

