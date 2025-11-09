use crate::context::AppContext;
use crate::user;
use rseata::{ RSEATA_CLIENT_SESSION};
use sea_orm::{
    ActiveModelTrait, ActiveValue, DbErr, EntityTrait, TransactionTrait,
};
use std::sync::Arc;

pub async fn get_user(
    app_ctx: Arc<AppContext>,
    user_id: i64,
) -> anyhow::Result<Option<user::user::Model>> {
    let session = RSEATA_CLIENT_SESSION.try_get().ok();
    tracing::info!("global session: {:?}", session);

    user::user::Entity::find_by_id(user_id)
        .one(&app_ctx.db_conn)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn add_user(
    app_ctx: Arc<AppContext>,
    add_user: user::user::Model,
) -> anyhow::Result<user::user::Model> {
    app_ctx
        .db_conn
        .transaction::<_, anyhow::Result<user::user::Model>, DbErr>(|txn| {
            Box::pin(async move {
                let one = user::user::Entity::find_by_id(add_user.id).one(txn).await?;
                let user = if one.is_none() {
                    let add = user::user::ActiveModel {
                        id: ActiveValue::set(add_user.id),
                        name: ActiveValue::set(add_user.name),
                        age: ActiveValue::set(add_user.age),
                        sex: ActiveValue::set(add_user.sex),
                    }
                    .insert(txn)
                    .await?;
                    Ok(add)
                } else {
                    Err(anyhow::anyhow!("User already exists"))
                };

                Ok::<_, DbErr>(user)
            })
        })
        .await?
}
