#![cfg(not(target_arch = "wasm32"))]

use async_graphql::{InputType};
use linera_sdk::base::{Amount, ApplicationId, Owner};
use linera_sdk::test::{ActiveChain, TestValidator};
use fungible::{Account, AccountOwner, FungibleTokenAbi, OperationOpt};

#[tokio::test]
async fn test_cross_chain_transfer() {
    // 合约部署时账户初始余额
    let initial_amount = Amount::from(1_000_000u128);
    // 转账余额
    let transfer_amount = Amount::from(50_000u128);
    // 模拟一个验证节点
    let (validator, bytecode_id) = TestValidator::with_current_bytecode().await;
    // 创建发送链
    let mut sender_chain = validator.new_chain().await;
    // 从发送链得到发送账户
    let sender_account = Owner::from(sender_chain.public_key());
    // 在发送链上部署我们的应用
    let application_id = sender_chain
        .create_application::<fungible::FungibleTokenAbi>(
            bytecode_id,
            (),
            initial_amount,
            vec![]
        ).await;

    // 创建接收链
    let receiver_chain = validator.new_chain().await;
    // 从接收链得到接收账户
    let receiver_account = Owner::from(receiver_chain.public_key());

    // 在发送链上创建一个区块并发送一笔交易
    sender_chain.add_block(|block| {
        block.with_operation(
            application_id,
            OperationOpt::Transfer {
                owner: AccountOwner::User(sender_account),
                amount: transfer_amount,
                target_account: Account {
                    chain_id: receiver_chain.id(),
                    owner: AccountOwner::User(receiver_account)
                }
            },
        );
    }).await;
    // 判断发送者余额是不是等于初始金额减去发送金额
    assert_eq!(
        query_account(application_id, &sender_chain, AccountOwner::User(sender_account)).await,
        Some(initial_amount.saturating_sub(transfer_amount))
    );
    // 让接收链接收消息
    receiver_chain.handle_received_messages().await;

    // 判断接收者余额是不是等于发送金额
    assert_eq!(
        query_account(application_id, &receiver_chain, AccountOwner::User(receiver_account)).await,
        Some(transfer_amount)
    )

}

async fn query_account(
    application_id: ApplicationId<FungibleTokenAbi>,
    chain: &ActiveChain,
    account_owner: AccountOwner
) -> Option<Amount> {
    let owner_value = match account_owner {
        AccountOwner::Application(id) => InputType::to_value(&id),
        AccountOwner::User(address) => InputType::to_value(&address)
    };
    // 注意：下面是两个大括号转译成一个实际的大括号，取值的两个大括号不需要转译
    let query = format!(
        "query {{ accounts(accountOwner: {{ User: {} }}) }}",
        owner_value
    );

    let value = chain.graphql_query(application_id, query).await;
    let balance = value.as_object()?.get("accounts")?.as_str()?;

    Some(balance.parse().unwrap())
}