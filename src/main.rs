// Copyright (c) 2025 Murtaza Shajapurwala
//
// Shazamq Operator - Kubernetes Operator for Shazamq Clusters

use futures::StreamExt;
use kube::{
    runtime::{controller::Action, Controller},
    Api, Client, ResourceExt,
};
use std::sync::Arc;
use tokio::time::Duration;
use tracing::{error, info};

mod crd;
mod reconciler;

use crd::ShazamqCluster;
use reconciler::Reconciler;

// Custom error type that implements std::error::Error
#[derive(Debug, thiserror::Error)]
enum ReconcilerError {
    #[error("Reconciliation failed: {0}")]
    ReconcileFailed(#[from] anyhow::Error),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
                async move { 
                    reconciler.reconcile(Arc::try_unwrap(obj).unwrap_or_else(|arc| (*arc).clone())).await
                        .map_err(ReconcilerError::from)
                }
            },
            |obj, error, _ctx| {
                error!(
                    name = obj.name_any(),
                    namespace = ?obj.namespace(),
                    error = %error,
                    "Reconciliation error"
                );
                Action::requeue(Duration::from_secs(60))
            },
            reconciler,
        )
        .for_each(|res| async move {
            match res {
                Ok((obj, _action)) => {
                    info!(
                        name = %obj.name,
                        namespace = ?obj.namespace,
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

