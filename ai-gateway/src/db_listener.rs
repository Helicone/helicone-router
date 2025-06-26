use futures::future::BoxFuture;
use meltdown::Token;
use sqlx::{
    PgPool,
    postgres::{PgListener, PgPoolOptions},
};
use tracing::{debug, error, info};

use crate::{
    config::database::DatabaseConfig,
    error::{init::InitError, runtime::RuntimeError},
};

/// A database listener service that handles LISTEN/NOTIFY functionality.
/// This service runs in the background and can be registered with meltdown.
#[derive(Debug, Clone)]
pub struct DatabaseListener {
    pool: PgPool,
}

impl DatabaseListener {
    pub async fn new(config: DatabaseConfig) -> Result<Self, InitError> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(config.acquire_timeout)
            .idle_timeout(config.idle_timeout)
            .max_lifetime(config.max_lifetime)
            .connect(&config.url)
            .await
            .map_err(|e| {
                error!(error = %e, "failed to create database pool");
                InitError::DatabaseConnection(e)
            })?;
        Ok(Self { pool })
    }

    /// Runs the database listener service.
    /// This includes listening for notifications and handling
    /// connection health.
    async fn run_service(&mut self) -> Result<(), RuntimeError> {
        info!("starting database listener service");

        // Create listener for LISTEN/NOTIFY
        let mut listener =
            PgListener::connect_with(&self.pool).await.map_err(|e| {
                error!(error = %e, "failed to create database listener");
                RuntimeError::Internal(
                    crate::error::internal::InternalError::Internal,
                )
            })?;

        // Listen for notifications on a channel (you can customize this)
        listener.listen("connected_cloud_gateways").await.map_err(|e| {
            error!(error = %e, "failed to listen on database notification channel");
            RuntimeError::Internal(crate::error::internal::InternalError::Internal)
        })?;

        // Process notifications
        loop {
            match listener.recv().await {
                Ok(notification) => {
                    debug!(
                        channel = notification.channel(),
                        payload = notification.payload(),
                        "received database notification"
                    );

                    // Handle the notification here
                    Self::handle_notification(&notification);
                }
                Err(e) => {
                    error!(error = %e, "error receiving database notification");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handles incoming database notifications.
    fn handle_notification(notification: &sqlx::postgres::PgNotification) {
        // Customize this method to handle different types of notifications
        info!(
            channel = notification.channel(),
            payload = notification.payload(),
            "processing notification"
        );

        // Example: You could dispatch to different handlers based on the
        // channel
        // TODO: Implement handle db listener
    }
}

impl meltdown::Service for DatabaseListener {
    type Future = BoxFuture<'static, Result<(), RuntimeError>>;

    fn run(mut self, mut token: Token) -> Self::Future {
        Box::pin(async move {
            tokio::select! {
                result = self.run_service() => {
                    if let Err(e) = result {
                        error!(error = %e, "database listener service encountered error, shutting down");
                    } else {
                        debug!("database listener service shut down successfully");
                    }
                    token.trigger();
                }
                () = &mut token => {
                    debug!("database listener service shutdown signal received");
                }
            }
            Ok(())
        })
    }
}
