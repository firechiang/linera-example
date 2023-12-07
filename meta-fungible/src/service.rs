#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use self::state::MetaFungible;
use async_trait::async_trait;
use linera_sdk::{base::WithServiceAbi, QueryContext, Service, ViewStateStorage};
use std::sync::Arc;
use async_graphql::{EmptySubscription, Schema};
use linera_sdk::graphql::GraphQLMutationRoot;
use thiserror::Error;
use meta_fungible::OperationPP;

linera_sdk::service!(MetaFungible);

impl WithServiceAbi for MetaFungible {
    type Abi = meta_fungible::MetaFungibleAbi;
}

#[async_trait]
impl Service for MetaFungible {
    type Error = ServiceError;
    type Storage = ViewStateStorage<Self>;

    async fn handle_query(
        self: Arc<Self>,
        _context: &QueryContext,
        query: Self::Query,
    ) -> Result<Self::QueryResponse, Self::Error> {
        let schema = Schema::build(self.clone(),OperationPP::mutation_root(),EmptySubscription).finish();
        let response = schema.execute(query).await;
        Ok(response)
    }
}

/// An error that can occur while querying the service.
#[derive(Debug, Error)]
pub enum ServiceError {
    /// Query not supported by the application.
    #[error("Queries not supported by application")]
    QueriesNotSupported,

    /// Invalid query argument; could not deserialize request.
    #[error("Invalid query argument; could not deserialize request")]
    InvalidQuery(#[from] serde_json::Error),

    // Add error variants here.
}
