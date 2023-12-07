#![cfg_attr(target_arch = "wasm32", no_main)]

use async_trait::async_trait;
use linera_sdk::{
    ApplicationCallResult,
    base::{SessionId, WithContractAbi}, CalleeContext, Contract, ExecutionResult, MessageContext,
    OperationContext, SessionCallResult, ViewStateStorage,
};
use linera_sdk::base::{Amount, ApplicationId, Owner};
use linera_sdk::contract::system_api;
use thiserror::Error;

use fungible::{Account, AccountOwner, ApplicationCallOpt, MessageOpt, OperationOpt};

use crate::state::InsufficientBalanceError;

use self::state::FungibleToken;

mod state;
/*
合约入口文件
*/

linera_sdk::contract!(FungibleToken);

impl WithContractAbi for FungibleToken {
    type Abi = fungible::FungibleTokenAbi;
}

/*
合约入口实现
*/
#[async_trait]
impl Contract for FungibleToken {
    // 指定合约自定义错误类型
    type Error = ContractError;
    // 指定合约数据存储实现
    type Storage = ViewStateStorage<Self>;

    // 合约部署时调用(注意： 我们指定了参数amount所以在部署合约的时候需要传递amount参数。--json-argument '"50000"')
    // 注意：参数_argument是Amount类型是因为我们在lib.rs文件里面将该类型指定为Amount
    async fn initialize(
        &mut self,
        _context: &OperationContext,
        _argument: Self::InitializationArgument,
    ) -> Result<ExecutionResult<Self::Message>, Self::Error> {
        log::debug!("{} 正在部署FungibleToken合约，初始余额 {}",_context.authenticated_signer.unwrap(),_argument);
        // 获取到调用合约签名者并且合约签名者不为空
        if let Some(owner) = _context.authenticated_signer {
            //let amount = Amount::from_str("50000").unwrap();
            // 给合约部署者加余额
            self.initialize_account(AccountOwner::User(owner), _argument).await;
        }
        Ok(ExecutionResult::default())
    }

    // 前端调用拦截转发
    async fn execute_operation(
        &mut self,
        context: &OperationContext,
        operation: Self::Operation,
    ) -> Result<ExecutionResult<Self::Message>, Self::Error> {
        // 匹配操作
        match operation {
            // 如果是转账操作
            OperationOpt::Transfer { owner, amount, target_account } => {
                // 验证签名
                Self::check_account_authentication(None,context.authenticated_signer, owner)?;
                // 减去自己账户余额
                self.debit(owner, amount).await?;
                // 返回完成转账信息
                Ok(self.finish_transfer_to_account(amount, target_account).await)
            }
            OperationOpt::Claim {source_account,amount,target_account} => {
                // 验证spource_account签名
                Self::check_account_authentication(None,context.authenticated_signer,source_account.owner)?;
                self.claim(source_account,amount,target_account).await
            }
        }
    }

    // 跨链相互调用
    async fn execute_message(
        &mut self,
        _context: &MessageContext,
        _message: Self::Message,
    ) -> Result<ExecutionResult<Self::Message>, Self::Error> {
        // 匹配操作
        match _message {
            MessageOpt::Credit { amount, owner } => {
                self.credit(owner, amount).await;
                Ok(ExecutionResult::default())
            }
            MessageOpt::Withdraw {owner,amount,target_account } => {
                // 验证owner签名
                Self::check_account_authentication(None,_context.authenticated_signer,owner)?;
                self.debit(owner,amount).await?;
                Ok(self.finish_transfer_to_account(amount,target_account).await)
            }
        }
    }
    // 相同链相互调用(注意：call的类型是ApplicationCallOpt，是因为我们在lib.rs文件里面把ApplicationCall的类型定义成了ApplicationCallOpt)
    async fn handle_application_call(
        &mut self,
        context: &CalleeContext,
        call: Self::ApplicationCall,
        _forwarded_sessions: Vec<SessionId>,
    ) -> Result<ApplicationCallResult<Self::Message, Self::Response, Self::SessionState>, Self::Error> {
        match call {
            ApplicationCallOpt::Balance {owner} => {
                let mut res = ApplicationCallResult::default();
                let balance = self.balance(&owner).await;
                // 注意：value是Amount类型是因为我们在lib.rs文件里面把Response的类型定义成了Amount
                res.value = balance;
                Ok(res)
            }
            ApplicationCallOpt::Transfer {owner,amount,target_account} => {
                // 验证发起者签名
                Self::check_account_authentication(context.authenticated_caller_id,context.authenticated_signer,owner)?;
                self.debit(owner,amount).await?;
                let mut res = ApplicationCallResult::default();
                let execution_res = self.finish_transfer_to_account(amount,target_account).await;
                res.execution_result = execution_res;
                Ok(res)
            }
            ApplicationCallOpt::Claim {source_account,amount,target_account} => {
                Self::check_account_authentication(context.authenticated_caller_id,context.authenticated_signer,source_account.owner)?;
                let mut res = ApplicationCallResult::default();
                let execution_res = self.claim(source_account,amount,target_account).await?;
                res.execution_result = execution_res;
                Ok(res)
            }
        }
    }

    async fn handle_session_call(
        &mut self,
        _context: &CalleeContext,
        _session: Self::SessionState,
        _call: Self::SessionCall,
        _forwarded_sessions: Vec<SessionId>,
    ) -> Result<SessionCallResult<Self::Message, Self::Response, Self::SessionState>, Self::Error> {
        Err(ContractError::SessionNotSupported)
    }
}

impl FungibleToken {
    // 验证签名或程序ID
    fn check_account_authentication(
        authenticated_application_id: Option<ApplicationId>,
        authenticated_signed: Option<Owner>,
        owner: AccountOwner) -> Result<(), ContractError> {
        match owner {
            // 签名者等于owner表示验证成功
            AccountOwner::User(address) if authenticated_signed == Some(address) => Ok(()),
            // 验证程序ID
            AccountOwner::Application(id) if authenticated_application_id == Some(id) => Ok(()),
            _ => Err(ContractError::IncorrectAuthentication)
        }
    }

    // 完成转账
    async fn finish_transfer_to_account(&mut self, amount: Amount, account: Account) -> ExecutionResult<MessageOpt> {
        // 如果账户的链ID等于当前链ID
        if account.chain_id == system_api::current_chain_id() {
            // 目标账户增加余额
            self.credit(account.owner, amount).await;
            ExecutionResult::default()
        } else {
            let message = MessageOpt::Credit {
                owner: account.owner,
                amount: amount,
            };
            // 不签名调用其它链或程序
            ExecutionResult::default().with_message(account.chain_id, message)
        }
    }
    // 获取代币
    async fn claim(&mut self,source_account: Account,amount:Amount,target_account:Account) -> Result<ExecutionResult<MessageOpt>,ContractError> {
        if source_account.chain_id == system_api::current_chain_id() {
            self.debit(source_account.owner,amount).await?;
            Ok(self.finish_transfer_to_account(amount,target_account).await)
        } else {
            let message = MessageOpt::Withdraw {
                owner: source_account.owner,
                amount: amount,
                target_account: target_account
            };
            // 签名调用其它链或程序
            Ok(ExecutionResult::default().with_authenticated_message(source_account.chain_id,message))
        }
    }
}

// 自定义一些合约错误
#[derive(Debug, Error)]
pub enum ContractError {
    #[error("Failed to deserialize BCS bytes")]
    BcsError(#[from] bcs::Error),

    #[error("Failed to deserialize JSON string")]
    JsonError(#[from] serde_json::Error),

    // 签名验证失败错误
    #[error("Incorrect Authentication")]
    IncorrectAuthentication,

    // 如果碰到InsufficientBalanceError错误会自动转成InsufficientBalance错误
    #[error("Insufficient Balance")]
    InsufficientBalance(#[from] InsufficientBalanceError),

    #[error("Session not supported")]
    SessionNotSupported,
}

/*
单元测试
*/
#[cfg(test)]
#[cfg(target_arch = "wasm32")]
pub mod tests {
    use std::str::FromStr;

    use futures::FutureExt;
    use linera_sdk::{Contract, OperationContext};
    use linera_sdk::base::{BlockHeight, ChainId};
    use linera_sdk::views::ViewStorageContext;
    use linera_views::views::{View, ViewError};
    use webassembly_test::webassembly_test;

    use super::*;

    #[webassembly_test]
    pub fn init() {
        let initial_amount = Amount::from_str("500000").unwrap();
        // 测试合约部署回调函数
        let result = create_and_init(initial_amount);
        if let Ok(fungible_token) = result {
            let account_owner = AccountOwner::User(creator());
            let balance = fungible_token.balance(&account_owner).now_or_never().unwrap();
            // 测试查询余额(注意：调用now_or_never()函数表示立即执行，如果不调用该函数则不会执行因为链上调用是异步的)
            assert_eq!(balance,initial_amount)
        }
    }

    fn create_and_init(amount: Amount) -> Result<FungibleToken, ViewError> {
        // 模拟创建键值存储
        linera_sdk::test::mock_key_value_store();
        // 拿到键值存储上下文
        let store = ViewStorageContext::default();
        // 给FungibleToken结构体对象加载 键值存储对象上下文(注意：FungibleToken本身就是ViewStorage因为代码上加了ViewStorageContext标识)
        // 注意：调用now_or_never()函数表示立即执行，如果不调用该函数则不会执行因为链上调用是异步的
        let execute_res = FungibleToken::load(store).now_or_never();
        if let Some(load_res) = execute_res {
            // Result有两个范型，？号表示直接取左边的数据，如果没取到就直接返回右边的数据(注意：这个？号表达式有个前提就是当前这个函数的返回值是Result类型)
            let mut fungible_token = load_res?;
            // 测试部署合约回调函数 initialize 的逻辑是否正确
            let result = fungible_token.initialize(&dummy_context(), amount).now_or_never().unwrap();
            assert!(result.is_ok());
            return Ok(fungible_token);
        } else {
            return Err(ViewError::NotFound(String::from("加载视图失败!")));
        }
    }

    /**
     * 模拟一个区块上下文
     */
    fn dummy_context() -> OperationContext {
        OperationContext {
            chain_id: ChainId([0; 4].into()),
            authenticated_signer: Some(creator()),
            height: BlockHeight(0),
            index: 0,
        }
    }

    fn creator() -> Owner {
        return "1c02a28d03e846b113de238d8880df3c9c802143b73aea5d173466701bee1786"
            .parse()
            .unwrap();
    }
}
