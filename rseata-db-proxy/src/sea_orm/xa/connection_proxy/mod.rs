mod impl_connection_trait;
mod impl_stream_trait;
mod impl_transaction_trait;

use sea_orm::error::*;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct XAId(pub String);

#[derive(Clone)]
pub struct XAConnectionProxy {
    pub url: String,
    pub sea_connection: sea_orm::DatabaseConnection,
}
impl XAConnectionProxy {
    pub async fn connect_mysql(url: &str) -> Result<Self, DbErr> {
        let t = sea_orm::Database::connect(url).await?;

        Ok(Self {
            url: url.to_string(),
            sea_connection: t,
        })
    }
}

impl Deref for XAConnectionProxy {
    type Target = sea_orm::DatabaseConnection;

    fn deref(&self) -> &Self::Target {
        &self.sea_connection
    }
}
impl DerefMut for XAConnectionProxy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sea_connection
    }
}

impl Debug for XAConnectionProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.sea_connection.fmt(f)
    }
}
 
