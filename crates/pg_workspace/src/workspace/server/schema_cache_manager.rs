use std::sync::{RwLock, RwLockReadGuard};

use pg_schema_cache::SchemaCache;
use sqlx::PgPool;

use crate::WorkspaceError;

use super::async_helper::run_async;

pub(crate) struct SchemaCacheHandle<'a> {
    inner: RwLockReadGuard<'a, SchemaCacheManagerInner>,
}

impl<'a> SchemaCacheHandle<'a> {
    pub(crate) fn new(cache: &'a RwLock<SchemaCacheManagerInner>) -> Self {
        Self {
            inner: cache.read().unwrap(),
        }
    }

    pub(crate) fn wrap(inner: RwLockReadGuard<'a, SchemaCacheManagerInner>) -> Self {
        Self { inner }
    }
}

impl AsRef<SchemaCache> for SchemaCacheHandle<'_> {
    fn as_ref(&self) -> &SchemaCache {
        &self.inner.cache
    }
}

#[derive(Default)]
pub(crate) struct SchemaCacheManagerInner {
    cache: SchemaCache,
    conn_str: String,
}

#[derive(Default)]
pub struct SchemaCacheManager {
    inner: RwLock<SchemaCacheManagerInner>,
}

impl SchemaCacheManager {
    pub fn load(&self, pool: PgPool) -> Result<SchemaCacheHandle, WorkspaceError> {
        let inner = self.inner.read().unwrap();

        if pool_to_conn_str(&pool) == inner.conn_str {
            Ok(SchemaCacheHandle::wrap(inner))
        } else {
            let new_conn_str = pool_to_conn_str(&pool);

            let maybe_refreshed = run_async(async move { SchemaCache::load(&pool).await })?;
            let refreshed = maybe_refreshed?;

            let mut inner = self.inner.write().unwrap();

            inner.cache = refreshed;
            inner.conn_str = new_conn_str;

            Ok(SchemaCacheHandle::new(&self.inner))
        }
    }
}

fn pool_to_conn_str(pool: &PgPool) -> String {
    let conn = pool.connect_options();

    match conn.get_database() {
        None => format!(
            "postgres://{}:<redacted_pw>@{}:{}",
            conn.get_username(),
            conn.get_host(),
            conn.get_port()
        ),
        Some(db) => format!(
            "postgres://{}:<redacted_pw>@{}:{}/{}",
            conn.get_username(),
            conn.get_host(),
            conn.get_port(),
            db
        ),
    }
}
