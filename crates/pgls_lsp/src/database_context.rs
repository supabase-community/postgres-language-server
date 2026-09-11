use pgls_configuration::{
    PartialConfiguration, PartialTypecheckConfiguration, StringSet,
    database::PartialDatabaseConfiguration,
};
use serde::{Deserialize, Deserializer};
use std::fmt;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SetDatabaseContextParams {
    pub(crate) context: Option<SessionDatabaseContext>,
}

pub(crate) struct SessionDatabaseContext {
    connection: SessionDatabaseConnection,
    search_path: Vec<String>,
}

struct SessionDatabaseConnection {
    host: String,
    port: u16,
    username: String,
    password: String,
    database: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SessionDatabaseContextWire {
    connection: SessionDatabaseConnection,
    search_path: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SessionDatabaseConnectionWire {
    host: String,
    port: u16,
    username: String,
    password: String,
    database: String,
}

impl<'de> Deserialize<'de> for SessionDatabaseContext {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = SessionDatabaseContextWire::deserialize(deserializer)?;
        if wire.search_path.is_empty() {
            return Err(serde::de::Error::custom(
                "searchPath must contain at least one schema name",
            ));
        }
        if wire
            .search_path
            .iter()
            .any(|schema| schema.trim().is_empty())
        {
            return Err(serde::de::Error::custom(
                "searchPath must contain only non-empty schema names",
            ));
        }
        Ok(Self {
            connection: wire.connection,
            search_path: wire.search_path,
        })
    }
}

impl<'de> Deserialize<'de> for SessionDatabaseConnection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = SessionDatabaseConnectionWire::deserialize(deserializer)?;
        if wire.host.trim().is_empty() {
            return Err(serde::de::Error::custom(
                "connection.host must be non-empty",
            ));
        }
        if wire.port == 0 {
            return Err(serde::de::Error::custom(
                "connection.port must be a non-zero u16",
            ));
        }
        if wire.username.trim().is_empty() {
            return Err(serde::de::Error::custom(
                "connection.username must be non-empty",
            ));
        }
        if wire.password.trim().is_empty() {
            return Err(serde::de::Error::custom(
                "connection.password must be non-empty",
            ));
        }
        if wire.database.trim().is_empty() {
            return Err(serde::de::Error::custom(
                "connection.database must be non-empty",
            ));
        }
        Ok(Self {
            host: wire.host,
            port: wire.port,
            username: wire.username,
            password: wire.password,
            database: wire.database,
        })
    }
}

impl SessionDatabaseContext {
    pub(crate) fn into_partial_configuration(self) -> PartialConfiguration {
        PartialConfiguration {
            db: Some(PartialDatabaseConfiguration {
                host: Some(self.connection.host),
                port: Some(self.connection.port),
                username: Some(self.connection.username),
                password: Some(self.connection.password),
                database: Some(self.connection.database),
                disable_connection: Some(false),
                ..Default::default()
            }),
            typecheck: Some(PartialTypecheckConfiguration {
                search_path: Some(StringSet::from_iter(self.search_path)),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

impl fmt::Debug for SetDatabaseContextParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SetDatabaseContextParams")
            .field("context", &self.context)
            .finish()
    }
}

impl fmt::Debug for SessionDatabaseContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionDatabaseContext")
            .field("connection", &self.connection)
            .field("search_path", &self.search_path)
            .finish()
    }
}

impl fmt::Debug for SessionDatabaseConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionDatabaseConnection")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("username", &self.username)
            .field("password", &"[redacted]")
            .field("database", &self.database)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::SetDatabaseContextParams;
    use serde_json::json;

    #[test]
    fn validates_the_strict_database_context_payload() {
        for payload in [
            json!({
                "context": {
                    "connection": {
                        "host": "",
                        "port": 5432,
                        "username": "analytics_user",
                        "password": "secret",
                        "database": "analytics"
                    },
                    "searchPath": ["public"]
                }
            }),
            json!({
                "context": {
                    "connection": {
                        "host": "localhost",
                        "port": 0,
                        "username": "analytics_user",
                        "password": "secret",
                        "database": "analytics",
                        "connectionString": "postgres://forbidden"
                    },
                    "searchPath": ["public"]
                }
            }),
            json!({
                "context": {
                    "connection": {
                        "host": "localhost",
                        "port": 5432,
                        "username": "analytics_user",
                        "password": "secret",
                        "database": "analytics",
                        "role": "forbidden"
                    },
                    "searchPath": ["public"]
                }
            }),
            json!({
                "context": {
                    "connection": {
                        "host": "localhost",
                        "port": 5432,
                        "username": "analytics_user",
                        "password": "secret",
                        "database": "analytics"
                    },
                    "searchPath": [""]
                }
            }),
            json!({
                "context": {
                    "connection": {
                        "host": "localhost",
                        "port": 5432,
                        "username": "analytics_user",
                        "password": "secret",
                        "database": "analytics"
                    },
                    "searchPath": []
                }
            }),
        ] {
            assert!(serde_json::from_value::<SetDatabaseContextParams>(payload).is_err());
        }
    }

    #[test]
    fn accepts_a_clear_context_and_redacts_password_from_debug() {
        let clear = serde_json::from_value::<SetDatabaseContextParams>(json!({
            "context": null
        }))
        .expect("clear context must deserialize");
        assert!(clear.context.is_none());

        let context = serde_json::from_value::<SetDatabaseContextParams>(json!({
            "context": {
                "connection": {
                    "host": "localhost",
                    "port": 5432,
                    "username": "analytics_user",
                    "password": "super-secret",
                    "database": "analytics"
                },
                "searchPath": ["tenant", "public"]
            }
        }))
        .expect("valid context must deserialize");
        let formatted = format!("{context:?}");
        assert!(formatted.contains("[redacted]"));
        assert!(!formatted.contains("super-secret"));
    }
}
