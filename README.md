#### 一、基础安装
```bash
# 安装Protobuf(Linera依赖)
$ sudo zypper install protobuf-devel
# 安装 Linera服务
$ cargo install linera-service
# 安装SDK主要用于单元测试(注意：写好了单元测试代码可使用命令 linera project test 执行单元测试)
$ cargo install linera-sdk
# 测试Linera是否安装成功
$ linera --help
```
#### 二、启动本地开发节点和部署项目(注意：以下命令都在项目目录下执行)
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
$ linera project publish-and-create --json-argument 'null'

# 启动一个带有前端的服务，用于和我们的本地链进行交互（注意：部署应用之前要同步账户就是上上一部的操作。服务的默认端口是8080，界面左边有使用文档）
$ linera service

# 访问http://localhost:8080服务输入如下GraphQL查询链上所有部署的应用信息
query {
  applications(chainId: "链ID") {
    id,
    link,
    description
  }
}
```


#### 三、使用Linera构建测试应用
```bash
# 创建一个名字叫xxxx的项目(注意：项目文件在当前目录下)
$ linera project new xxxx
# 编译项目
$ cargo build
# 测试应用
$ linera project test
```