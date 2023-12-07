#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use self::state::MetaFungible;
use async_trait::async_trait;
use linera_sdk::{
    base::{SessionId, WithContractAbi},
    ApplicationCallResult, CalleeContext, Contract, ExecutionResult, MessageContext,
    OperationContext, SessionCallResult, ViewStateStorage,
};
use linera_sdk::base::ApplicationId;
use thiserror::Error;
use meta_fungible::OperationPP;

linera_sdk::contract!(MetaFungible);

impl WithContractAbi for MetaFungible {
    type Abi = meta_fungible::MetaFungibleAbi;
}

impl MetaFungible {
    fn fungible_id() -> Result<ApplicationId<fungible::FungibleTokenAbi>,ContractError> {
        Self::parameters()
    }
}

#[async_trait]
impl Contract for MetaFungible {
    type Error = ContractError;
    type Storage = ViewStateStorage<Self>;

    async fn initialize(
        &mut self,
        _context: &OperationContext,
        _argument: Self::InitializationArgument,
    ) -> Result<ExecutionResult<Self::Message>, Self::Error> {
        Ok(ExecutionResult::default())
    }
    // 前端调用拦截转发
    async fn execute_operation(
        &mut self,
        _context: &OperationContext,
        _operation: Self::Operation,
    ) -> Result<ExecutionResult<Self::Message>, Self::Error> {
        match _operation {
            OperationPP::Transfer {owner,amount,target_account} => {
                // 装配调用另一个程序的参数
                let transfer_call = fungible::ApplicationCallOpt::Transfer {
                    owner,
                    amount,
                    target_account
                };
                // 调用另一个程序
                self.call_application(true,Self::fungible_id()?,&transfer_call,vec![]).await?;
            }
        }
        Ok(ExecutionResult::default())
    }

    async fn execute_message(
        &mut self,
        _context: &MessageContext,
        _message: Self::Message,
    ) -> Result<ExecutionResult<Self::Message>, Self::Error> {
        Ok(ExecutionResult::default())
    }

    async fn handle_application_call(
        &mut self,
        _context: &CalleeContext,
        _call: Self::ApplicationCall,
        _forwarded_sessions: Vec<SessionId>,
    ) -> Result<ApplicationCallResult<Self::Message, Self::Response, Self::SessionState>, Self::Error>
    {
        Ok(ApplicationCallResult::default())
    }

    async fn handle_session_call(
        &mut self,
        _context: &CalleeContext,
        _session: Self::SessionState,
        _call: Self::SessionCall,
        _forwarded_sessions: Vec<SessionId>,
    ) -> Result<SessionCallResult<Self::Message, Self::Response, Self::SessionState>, Self::Error>
    {
        Ok(SessionCallResult::default())
    }
}

/// An error that can occur during the contract execution.
#[derive(Debug, Error)]
pub enum ContractError {
    /// Failed to deserialize BCS bytes
    #[error("Failed to deserialize BCS bytes")]
    BcsError(#[from] bcs::Error),

    /// Failed to deserialize JSON string
    #[error("Failed to deserialize JSON string")]
    JsonError(#[from] serde_json::Error),

    // Add more error variants here.
}
