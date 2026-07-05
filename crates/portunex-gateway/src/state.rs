//! 共享应用状态,注入到所有 handler。

use std::sync::Arc;
use crate::config::Settings;

#[derive(Clone)]
pub struct AppState {
    inner: Arc<Inner>,
}

pub struct Inner {
    pub settings: Settings,
    pub pool: sqlx::PgPool,
    // TODO: redis 客户端、clickhouse 客户端、upstream 池、路由器(scheduler)、
    //       provider 缓存等,按原 services/* 逐步接入。
}

impl AppState {
    pub fn new(settings: Settings, pool: sqlx::PgPool) -> Self {
        Self { inner: Arc::new(Inner { settings, pool }) }
    }
    pub fn settings(&self) -> &Settings { &self.inner.settings }
    pub fn pool(&self) -> &sqlx::PgPool { &self.inner.pool }
}
