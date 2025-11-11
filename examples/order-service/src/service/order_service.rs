use crate::context::AppContext;
use crate::order;
use anyhow::anyhow;
use example_proto::rseata_proto::proto::{AddUserRequest, GetUserByIdRequest};
use rseata::FutureExt;
use rseata::RSEATA_CLIENT_SESSION;
use rseata::global_transaction;
use sea_orm::sqlx::types::uuid;
use sea_orm::{
    ActiveModelTrait, ActiveValue, DbErr, EntityTrait, TransactionSession, TransactionTrait,
};
use std::sync::Arc;

#[global_transaction("add_order_then_add_user")]
pub async fn add_order_then_add_user(app_ctx: Arc<AppContext>) -> anyhow::Result<()> {
    let session = RSEATA_CLIENT_SESSION.try_get().ok();
    tracing::info!("start transaction session is : {:?}", session);

    let db = app_ctx.db_conn.clone();
    let a = db
        .clone()
        .transaction::<_, (), DbErr>(|txn| {
            Box::pin(async move {
                let session = RSEATA_CLIENT_SESSION.try_get().ok();
                tracing::info!("in transaction session is : {:?}", session);
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
                print!("user_client add  user {:?}", user);
                Ok::<_, DbErr>(())
            })
        })
        .await?;

    let session = RSEATA_CLIENT_SESSION.try_get().ok();
    tracing::info!("end transaction session is : {:?}", session);

    Ok(())
}
