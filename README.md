**RSeata - Rust实现的分布式事务框架**

**简介**

RSeata是一个基于Rust语言的分布式事务解决方案，旨在以高性能和简单易用的方式处理微服务架构下的分布式事务问题。该项目灵感来自于Seata，支持AT模式。

1. 支持AT,XA模式。（TODO：TCC，SAGE）
2. 与 SeaORM / Diesel 集成，支持MySQL和PostgresSQL
3. 支持gRPC微服务之间的分布式事务
4. 通过注解声明全局事务
5. 基于Tonic的gRPC拦截器，自动传播事务上下文
6. 包含TC,TM,RM (概念参考Seata)
   1. TC (Transaction Coordinator) - 事务协调者,维护全局和分支事务的状态，驱动全局事务提交或回滚。
   2. TM (Transaction Manager) - 事务管理器,定义全局事务的范围：开始全局事务、提交或回滚全局事务。
   3. RM (Resource Manager) - 资源管理器 管理分支事务处理的资源，与TC交谈以注册分支事务和报告分支事务的状态，并驱动分支事务提交或回滚。

**项目结构**
* rseata-core: 核心库，包含事务上下文、全局事务钩子等。
* rseata-tm: 事务管理器。
* rseata-rm: 资源管理器。
* rseata-db-proxy: 数据库代理，支持SeaORM和Diesel（计划中）。
* rseata-micro: 微服务支持，包括gRPC拦截器和宏。
* rseata-proto: gRPC协议定义。
* examples: 示例代码，包括order-service和user-service。

**快速开始**

**前提条件**
* Rust 1.90或更高版本
* MySQL 5.7或更高版本（用于示例）

**运行示例**
   1. 克隆项目
      `git clone https://github.com/your-username/rseata.git`

      cd rseata
   2. 设置数据库

      创建两个数据库：order和user
      * 运行示例中的SQL脚本
        1. examples/user-service/user.sql
        2. examples/user-service/user.sql
      * 数据库连接环境变量：
        1. USER_DATABASE_URL=mysql://root:root@127.0.0.1:3306/user
        2. ORDER_DATABASE_URL=mysql://root:root@127.0.0.1:3306/order
    
   3. 运行
      1. **启动TC**  `cd rseata-tc` `cargo run`
      2. `cd examples/user-service`
         `cargo run`
      3. `cd examples/order-service`
         `cargo run`
   4. 测试分布式事务
      * 通过order-service的API创建订单，order-service会调用user-service添加用户，这两个操作会在一个全局事务中。
      * http://127.0.0.1:4002/add_order_then_add_user


**添加依赖**
在您的Cargo.toml中添加：
```toml 
[dependencies]
rseata = { version = "0", features = ["full"] }
```
**配置**
* 需要配置数据库连接和gR服务端点。参考示例中的.env文件。

**初始化**
```
 rseata::init().await;
```
**定义数据库实体（使用SeaORM）**

```rust
use rseata::global_transaction;

#[global_transaction("your_transaction_name")]
pub async fn add_order_then_add_user(app_ctx: Arc<AppContext>) -> anyhow::Result<()> {
    let db = app_ctx.db_conn.clone();
    db.transaction::<_, (), DbErr>(|txn| {
            Box::pin(async move {
                
                // local transaction
                order::order::Entity::insert(new_order).exec(txn).await?;
 
                // grpc 
                let user = app_ctx
                    .user_client
                    .get()
                    .await
                    .map_err(|e| DbErr::Custom(e.to_string()))?
                    .user
                    .add_user(AddUserRequest {
                        name: "".to_string(),
                        age: None,
                        sex: None,
                    })
                    .await
                    .map_err(|e| DbErr::Custom(e.to_string()))?;
                
                Ok::<_, DbErr>(())
            })
        })
        .await?;

    Ok(())
}
```

**使用数据库代理**
* 在SeaORM的连接上，使用ConnectionProxy来包装，以便于分支事务的注册。

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
   dotenv::dotenv().ok();
   rseata::init().await;
   
   let connect_url = dotenv::var("ORDER_DATABASE_URL")
           .unwrap_or("mysql://root:root@127.0.0.1:3306/order".to_string());
   let conn = rseata::db_proxy::sea_orm::ConnectionProxy::connect(&connect_url).await?;
 
   Ok(())
}
```
**在gRPC Client中，使用提供的拦截器（RseataInterceptor）来传播事务上下文。**
```rust
let grpc_client = UserServiceClient::with_interceptor(channel, RseataInterceptor);
```
**在gRPC服务中，使用提供的拦截器（SeataMiddlewareLayer）来传播事务上下文。**
```rust
pub(crate) async fn start_grpc_server(ctx: Arc<AppContext>) -> anyhow::Result<()> {
    let addr = std::env::var("GRPC_BIND")
        .unwrap_or_else(|_| "0.0.0.0:9001".into())
        .parse()?;
    tracing::info!("Server started on 0.0.0.0:9001");
    Server::builder()
        .layer(SeataMiddlewareLayer) // 在gRPC服务中，使用提供的拦截器（SeataMiddlewareLayer）来传播事务上下文。
        .add_service(UserServiceServer::new(UserGrpcService { app_ctx: ctx }))
        .serve(addr)
        .await?;
    Ok(())
}
```

## 🙏 致谢

感谢 Seata 项目提供的设计灵感

感谢 Tonic 提供的 gRPC 框架

感谢 SeaORM 提供的 ORM 支持



