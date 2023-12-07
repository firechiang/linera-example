#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use self::state::FungibleToken;
use async_trait::async_trait;
use linera_sdk::{base::WithServiceAbi, QueryContext, Service, ViewStateStorage};
use std::sync::Arc;
use async_graphql::{EmptySubscription, Object, Response, Schema};
use linera_sdk::base::{Amount};
use thiserror::Error;
use fungible::{Account, AccountOwner, OperationOpt};

/*
查询服务相关实现(就是定义一些只读的数据给前端调用)
*/

linera_sdk::service!(FungibleToken);

impl WithServiceAbi for FungibleToken {
    type Abi = fungible::FungibleTokenAbi;
}

#[async_trait]
impl Service for FungibleToken {
    // 指定合约自定义错误类型(就是查询错误类型)
    type Error = ServiceError;
    // 指定合约数据存储实现
    type Storage = ViewStateStorage<Self>;

    async fn handle_query(
        self: Arc<Self>,
        _context: &QueryContext,
        request: Self::Query,
    ) -> Result<Response, Self::Error> {
        // 注意：这个MutationRoot可以使用GraphQLMutationRoot注解生成，具体可以参考meta-fungible模块lib.rs文件OperationPP枚举，然后看service.rs怎么使用
        let schema = Schema::build(self.clone(),MutationRoot {},EmptySubscription).finish();
        let response = schema.execute(request).await;
        return Ok(response);
    }
}

struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn transfer(&self,owner: AccountOwner,amount: Amount,target_account: Account) -> Vec<u8> {
        // 直接将 OperationOpt::Transfer 序列化成Byte数组
        bcs::to_bytes(&OperationOpt::Transfer {owner,amount,target_account}).unwrap()
    }
}


#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Queries not supported by application")]
    QueriesNotSupported,

    #[error("Invalid query argument; could not deserialize request")]
    InvalidQuery(#[from] serde_json::Error),
}
