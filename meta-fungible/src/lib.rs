use async_graphql::{Request, Response};
use linera_sdk::base::{Amount, ApplicationId, ContractAbi, ServiceAbi};
use linera_sdk::graphql::GraphQLMutationRoot;
use serde::{Deserialize, Serialize};
use fungible::{Account, AccountOwner};

pub struct MetaFungibleAbi;

impl ContractAbi for MetaFungibleAbi {
    // 参数（注意：这个）
    type Parameters = ApplicationId<fungible::FungibleTokenAbi>;
    type InitializationArgument = ();
    type Operation = OperationPP;
    type Message = ();
    type ApplicationCall = ();
    type SessionCall = ();
    type SessionState = ();
    type Response = ();
}

impl ServiceAbi for MetaFungibleAbi {
    type Parameters = ();
    type Query = Request;
    type QueryResponse = Response;
}
// GraphQLMutationRoot是让GraphQL可以识别到该枚举
#[derive(Debug,Serialize,Deserialize,GraphQLMutationRoot)]
pub enum OperationPP {
    Transfer {
        owner: AccountOwner,
        amount: Amount,
        target_account: Account
    }
}
