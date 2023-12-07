use linera_sdk::base::{Amount};
use linera_sdk::views::{MapView, ViewStorageContext};
use linera_views::views::{GraphQLView, RootView};
use thiserror::Error;
use fungible::AccountOwner;

/*
状态文件定义数据结构以及应用核心逻辑实现
*/

#[derive(RootView, GraphQLView)]
#[view(context = "ViewStorageContext")]
pub struct FungibleToken {
    /* 记录账户余额（注意：可以是地址账户也可以是程序账户） */
    pub accounts: MapView<AccountOwner,Amount>,
}


// 定义余额不足异常
#[derive(Clone, Copy, Debug, Error)]
#[error("Insufficient Balance")]
pub struct InsufficientBalanceError;

#[allow(dead_code)]//该注解用于去除未使用警告
impl FungibleToken {
    // 初始账户
    pub async fn initialize_account(&mut self, owner: AccountOwner, amount: Amount) {
        self.accounts
            .insert(&owner, amount)
            .expect("账户余额记录失败!");
    }
    // 获取账户余额
    pub async fn balance(&self, account: &AccountOwner) -> Amount {
        return self.accounts
            .get(account)
            .await
            .expect("获取账户余额失败!")
            .unwrap_or_default();
    }
    // 给某个账户加余额
    pub async fn credit(&mut self, account: AccountOwner, amount: Amount) {
        let mut balance = self.balance(&account).await;
        // 余额相加并防止溢出
        balance.saturating_add_assign(amount);
        self.accounts
            .insert(&account, balance)
            .expect("增加更新余额失败!");
    }

    // 给账户减余额
    pub async fn debit(&mut self, account: AccountOwner, amount: Amount) -> Result<(), InsufficientBalanceError> {
        let mut balance = self.balance(&account).await;
        // 余额相减如果不足抛出 InsufficientBalanceError
        balance.try_sub_assign(amount).map_err(|_| InsufficientBalanceError)?;
        self.accounts
            .insert(&account, balance)
            .expect("减少更新余额失败!");
        Ok(())
    }
}
