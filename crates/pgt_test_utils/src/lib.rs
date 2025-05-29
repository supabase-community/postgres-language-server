pub mod test_database;

pub static MIGRATIONS: sqlx::migrate::Migrator = sqlx::migrate!("./testdb_migrations");
