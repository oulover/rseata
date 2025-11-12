<div align="center">

  <h1>RSeata</h1>

<h3>🐚Rust实现的分布式事务框架</h3>

[![crate](https://img.shields.io/badge/crates.io-rseata-0)](https://crates.io/crates/rseata)
[![crate](https://img.shields.io/badge/doc-rseata-0)](https://docs.rs/rseata)

</div>

# Rseata

**简介**

RSeata是一个基于Rust语言的分布式事务解决方案，旨在以高性能和简单易用的方式处理微服务架构下的分布式事务问题。该项目灵感来自于Seata，参考了Seata的核心架构，目前支持AT、XA模式。

* 项目暂未全部完善，欢迎提出意见或建议！

### 特性

1. 支持AT,XA模式。(计划中：TCC,SAGA)
2. 支持Sea-orm集成。(计划中：Diesel)
3. 支持Mysql。(计划中：Postgresql)
4. 通过注解声明全局事务。
5. 基于Tonic的gRPC拦截器，自动传播事务上下文。
6. 包含TC,TM,RM (概念参考Seata)
    1. TC (Transaction Coordinator) - 事务协调器,维护全局和分支事务的状态，驱动全局事务提交或回滚。
    2. TM (Transaction Manager) - 事务管理器,定义全局事务的范围：开始全局事务、提交或回滚全局事务。
    3. RM (Resource Manager) - 资源管理器 管理分支事务处理的资源，与TC交谈以注册分支事务和报告分支事务的状态，并驱动分支事务提交或回滚。

## 快速开始

+ [集成示例：axum、tonic grpc、sea-orm、mysql、rseata、xa模式](https://github.com/oulover/rseata/tree/master/examples)

### 前提条件

* Rust 1.90或更高版本
* MySQL 5.7或更高版本（用于示例）

### 使用示例

* axum、tonic grpc、sea-orm、mysql、rseata、xa模式

#### TC server

1. 启动TC server
    ```shell
     git clone https://github.com/oulover/rseata.git
     cd rseata
     cargo run
    ```

#### TM RM

1. 添加依赖
   ```toml
    rseata = "0.1.2"
   ```
2. 环境变量
   ```toml
   # TM
   RSEATA_TC_GRPC_IP=127.0.0.1
   RSEATA_TC_GRPC_PROT=9811
   RSEATA_TM_APPLICATION_ID=order
   RSEATA_TM_TRANSACTION_SERVICE_GROUP=order_group
   
   # RM
   RSEATA_RM_RESOURCE_GROUP_ID=order_group
   RSEATA_RM_RESOURCE_ID=order
   ```
3. 设置grpc拦截器
    * 环境变量：
    * GRPC client：使用 **RseataInterceptor** 传播事务上下文
         ```rust
            #[tokio::main]
            async fn main() -> anyhow::Result<()> {
                let channel = Endpoint::from_str(&"tcp://127.0.0.1:8001")?.connect().await?;
                // use rseata_core::grpc_client::RseataInterceptor
                let client = UserServiceClient::with_interceptor(channel, rseata_core::grpc_client::RseataInterceptor);
            }
         ```
    * GRPC service：使用 **SeataMiddlewareLayer** 传播事务上下文
         ```rust
             async fn main() -> anyhow::Result<()> {
                 tonic::transport::Server::builder()
                     .layer(SeataMiddlewareLayer) // 使用 SeataMiddlewareLayer 传播事务上下文
                     .add_service(UserServiceServer::new())
                     .serve(addr)
                     .await?;
                 Ok(())
             }
         ```
4. 数据库代理：
   **mysql sea-orm** 示例 XAConnectionProxy 实现了sea-orm 同样的trait

    ```rust
            #[tokio::main]
            async fn main() -> anyhow::Result<()> {
            
                rseata::init().await;// must init
                //  DATABASE_URL : mysql://root:root@127.0.0.1:3306/user
                let connect_url = dotenv::var("DATABASE_URL").unwrap();
                let conn = rseata::db_proxy::sea_orm::XAConnectionProxy::connect_mysql(&connect_url).await?;
                
                Ok(())
            }
    ```

5. 全局事务注解
   ```rust
            #[global_transaction("add_order_then_add_user")] // 开启全局事务
            pub async fn add_order_then_add_user(db_conn: XAConnectionProxy) -> anyhow::Result<()> {
                // 实现了sea-orm 同样的trait，使用和 sea-orm 一样
                db_conn
                    .transaction::<_, (), DbErr>(|txn| {
                        Box::pin(async move {
                           
                            let order_id = uuid::Uuid::new_v4().as_u128() as i64;
                            let old_order = order::order::Entity::find_by_id(order_id).one(txn).await?;
            
                            if old_order.is_none() {
                                let new_order = order::order::ActiveModel {
                                    id: ActiveValue::set(order_id),
                                    product: ActiveValue::set(String::from(uuid::Uuid::new_v4())),
                                    count: ActiveValue::set(Some(11)),
                                    amount: ActiveValue::set(Some(22)),
                                };
                                order::order::Entity::insert(new_order).exec(txn).await?;
                            }
                            
                            // grpc 调用 会通过grpc拦截器 传播事务上下文
                            let user = 
                                user_grpc_client
                                .add_user(AddUserRequest {
                                    name: "".to_string(),
                                    age: None,
                                    sex: None,
                                })
                                .await
                                .map_err(|e| DbErr::Custom(e.to_string()))?;
                            print!("user_client add  user {:?}", user);
                            Ok::<_, DbErr>(())
                        })
                    })
                    .await?;
            
                let session = RSEATA_CLIENT_SESSION.try_get().ok();
                tracing::info!("end transaction session is : {:?}", session);
            
                Ok(())
            }
    ```

## 项目结构

* rseata-core: 核心库，包含事务上下文，全局事务钩子等。
* rseata-tc: 事务协调器。
* rseata-tm: 事务管理器。
* rseata-rm: 资源管理器。
* rseata-db-proxy: 数据源代理。
* rseata-micro: 微服务支持，包括gRPC拦截器和宏。
* rseata-proto: gRPC协议定义。
* rseata-error: 错误处理。
* examples: 示例代码，包括order-service和user-service。

## 🙏 致谢

感谢 Seata 项目提供的设计灵感

感谢 Tonic 提供的 gRPC 框架

感谢 SeaORM 提供的 ORM 支持



