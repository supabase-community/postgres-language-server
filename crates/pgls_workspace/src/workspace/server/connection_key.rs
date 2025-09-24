use sqlx::PgPool;

use crate::settings::DatabaseSettings;

/// A unique identifier for database connection settings
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct ConnectionKey {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub database: String,
}

impl From<&DatabaseSettings> for ConnectionKey {
    fn from(settings: &DatabaseSettings) -> Self {
        Self {
            host: settings.host.clone(),
            port: settings.port,
            username: settings.username.clone(),
            database: settings.database.clone(),
        }
    }
}

impl From<&PgPool> for ConnectionKey {
    fn from(pool: &PgPool) -> Self {
        let conn = pool.connect_options();

        match conn.get_database() {
            None => Self {
                host: conn.get_host().to_string(),
                port: conn.get_port(),
                username: conn.get_username().to_string(),
                database: String::new(),
            },
            Some(db) => Self {
                host: conn.get_host().to_string(),
                port: conn.get_port(),
                username: conn.get_username().to_string(),
                database: db.to_string(),
            },
        }
    }
}
