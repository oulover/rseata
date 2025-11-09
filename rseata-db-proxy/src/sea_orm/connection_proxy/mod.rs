mod impl_connection_trait;
mod impl_stream_trait;
mod impl_transaction_trait;

use sea_orm::error::*;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

#[derive(Clone)]
pub struct ConnectionProxy {
    url: String,
    inner: sea_orm::DatabaseConnection,
}
impl ConnectionProxy {
    pub async fn connect(url: &str) -> Result<Self, DbErr> {
        let t = sea_orm::Database::connect(url).await?;
        Ok(Self {
            url: url.to_string(),
            inner: t,
        })
    }
}
impl Deref for ConnectionProxy {
    type Target = sea_orm::DatabaseConnection;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl DerefMut for ConnectionProxy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Debug for ConnectionProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.inner.fmt(f)
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
