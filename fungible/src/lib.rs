use async_graphql::{InputObject, Request, Response, scalar};
use linera_sdk::base::{Amount, ApplicationId, ChainId, ContractAbi, Owner, ServiceAbi};
use serde::{Deserialize, Serialize};

/*
合约上下文相关定义文件
*/

pub struct FungibleTokenAbi;

// 合约写入相关Abi
impl ContractAbi for FungibleTokenAbi {
    type Parameters = ();
    // 合约部署时的参数
    type InitializationArgument = Amount;
    // 操作类型
    type Operation = OperationOpt;
    // 消息类型
    type Message = MessageOpt;
    type ApplicationCall = ApplicationCallOpt;
    type SessionCall = ();
    type SessionState = ();
    type Response = Amount;
}

// 合约查询相关Abi
impl ServiceAbi for FungibleTokenAbi {
    type Parameters = ();
    type Query = Request;
    type QueryResponse = Response;
}

// 相同链程序相互调用操作枚举
#[derive(Debug,Deserialize,Serialize)]
pub enum ApplicationCallOpt {
    Balance {
        owner: AccountOwner
    },
    Transfer {
        owner: AccountOwner,
        amount: Amount,
        target_account: Account
    },
    Claim {
        source_account: Account,
        amount: Amount,
        target_account: Account
    }
}

// 操作枚举(注意：操作相关用于前端调用)
#[derive(Debug, Deserialize, Serialize)]
pub enum OperationOpt {
    // 转账
    Transfer {
        owner: AccountOwner,
        amount: Amount,
        target_account: Account,
    },
    // 获取代币
    Claim {
        source_account: Account,
        amount: Amount,
        target_account: Account
    }

}

// 消息枚举(注意：消息相关操作用于内部调用就是链上程序相互调用)
#[derive(Debug, Deserialize, Serialize)]
pub enum MessageOpt {
    // 增加余额
    Credit {
        amount: Amount,
        owner: AccountOwner,
    },
    // 提取
    Withdraw {
        owner: AccountOwner,
        amount: Amount,
        target_account: Account
    }
}

// 这个是graphql解析使用
scalar!(AccountOwner);

#[derive(Clone,Copy,Debug,Deserialize,Eq,Ord,PartialEq,PartialOrd,Serialize)]
pub enum AccountOwner {
    User(Owner),
    Application(ApplicationId)
}

/*
账户信息
*/
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize,InputObject)]
pub struct Account {
    pub chain_id: ChainId,
    pub owner: AccountOwner,
}
