mod impl_connection_trait;
mod impl_stream_trait;
mod impl_transaction_trait;
mod impl_branch_transaction;

use sea_orm::error::*;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

#[derive(Clone)]
pub struct ATConnectionProxy {
    pub url: String,
    pub sea_conn: sea_orm::DatabaseConnection,
}
impl ATConnectionProxy {
    pub async fn connect_mysql(url: &str) -> Result<Self, DbErr> {
        let t = sea_orm::Database::connect(url).await?;
        Ok(Self {
            url: url.to_string(),
            sea_conn: t,
        })
    }
}
impl Deref for ATConnectionProxy {
    type Target = sea_orm::DatabaseConnection;

    fn deref(&self) -> &Self::Target {
        &self.sea_conn
    }
}
impl DerefMut for ATConnectionProxy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sea_conn
    }
}

impl Debug for ATConnectionProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.sea_conn.fmt(f)
    }
}

pub fn get_url(url: &str) -> String {
    if url.contains("?") {
        let r = url.split("?");
        r.collect::<Vec<&str>>()[0].to_string()
    } else {
        url.to_string()
    }
}
