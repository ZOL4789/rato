# rato

## 一、介绍
    rato是以rust、axum、sea_orm、mysql、redis开发的支持权限验证的web应用。
## 二、依赖
    rust：1.81.0
    axum: 0.8.1
    sea_orm: 1.1.8
    mysql：8.0
    redis：3.2.10
## 三、启动
### 3.1 安装rust
    参考：https://www.rust-lang.org/
### 3.2 拉取代码
    git clone https://github.com/ZOL4789/rato.git
    cd rato
### 3.3 安装sea-orm-cli（非必须）
    cargo install sea-orm-cli
### 3.4 执行db.sql
### 3.5 修改配置
    找到.env文件，修改
    DATABASE_URL=mysql://用户名:密码@localhost:端口/数据库
    REDIS_URL=redis://default:密码@127.0.0.1
### 3.6 运行
    cargo run --bin rato
### 3.7 打包
    cargo build --release --bin rato