#### 一、部署应用（注意：该应用部署是建立在fungible程序已经部署好了的情况下，因为该程序是去调用fungible程序。在部署的过程但中如果提示钱包文件被锁定了，就把测试服务窗口关了）
```bash
# 启动本地节点（注意：启动本地节点会创建两个数据文件如下，复制下来后面要用）
# export LINERA_WALLET="/tmp/.tmpueqfPc/wallet_0.json"
# export LINERA_STORAGE="rocksdb:/tmp/.tmpueqfPc/client_0.db"
$ linera net up

# 注意：另开一个命令行窗口来执行下面的命令
# 创建钱包临时变量（注意：这两个临时变量所指向的数据文件是上一步启动本地节点时自动创建的）
$ export LINERA_WALLET="/tmp/.tmpueqfPc/wallet_0.json"
$ export LINERA_STORAGE="rocksdb:/tmp/.tmpueqfPc/client_0.db"
# 同步账户
$ linera sync-balance
# 查看钱包信息（注意：绿色标记chainId是默认链）
$ linera wallet show

# 部署当前目录下项目到链上(参数传的是null)（注意：部署应用之前要同步账户就是上一部的操作。部署完成后会显示项目地址）
# 注意：要先把fungible程序部署好，因为部署当前程序需要fungible程序ID
$ linera project publish-and-create \
--required-application-ids "fungible程序ID" \
--json-parameters '"fungible程序ID"'
```


#### 二、启动测试服务并测试
```bash
# 启动一个带有前端的服务，用于和我们的本地链进行交互（注意：部署应用之前要同步账户就是上上一部的操作。服务的默认端口是8080，界面左边有使用文档）
# 访问http://localhost:8080服务输入如下GraphQL查询链上所有部署的应用信息
$ linera service

# 访问http://localhost:8080服务输入如下GraphQL查询链上所有部署的应用信息，找到刚刚我们部署的应用
query {
  applications(chainId: "链ID") {
    id,
    link,
    description
  }
}

# 通过上面的查询，从结果中找到link属性复制其内容，如下，在浏览器上新建一个窗口打开
# http://localhost:8080/chains/e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65/applications/程序ID

# 测试转账（注意：转账成功后，再回到fungible程序查询界面，看看那个有钱的用户地址的钱是不是少了）
mutation {
  transfer(
    owner: {
      User: "fungible程序有钱的那个用户地址"
    },
    amount: "500",
    targetAccount: {
      chainId: "链ID",
      owner: {
        User: "2b04c5e200b1c0ee70beb5f5cd7eddebc2d94647ddc7ad59d851f5f1b3cdbecc"
      }
    }
  )
}
```