use crate::schema_cache::SchemaCacheItem;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnClassKind {
    OrdinaryTable,
    View,
    MaterializedView,
    ForeignTable,
    PartitionedTable,
}

impl From<&str> for ColumnClassKind {
    fn from(value: &str) -> Self {
        match value {
            "r" => ColumnClassKind::OrdinaryTable,
            "v" => ColumnClassKind::View,
            "m" => ColumnClassKind::MaterializedView,
            "f" => ColumnClassKind::ForeignTable,
            "p" => ColumnClassKind::PartitionedTable,
            _ => panic!(
                "Columns belonging to a class with pg_class.relkind = '{}' should be filtered out in the query.",
                value
            ),
        }
    }
}

impl From<String> for ColumnClassKind {
    fn from(value: String) -> Self {
        ColumnClassKind::from(value.as_str())
    }
}

impl From<char> for ColumnClassKind {
    fn from(value: char) -> Self {
        ColumnClassKind::from(String::from(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    pub name: String,

    pub table_name: String,
    pub table_oid: i64,
    /// What type of class does this column belong to?
    pub class_kind: ColumnClassKind,

    pub schema_name: String,
    pub type_id: i64,
    pub is_nullable: bool,

    pub is_primary_key: bool,
    pub is_unique: bool,

    /// The Default "value" of the column. Might be a function call, hence "_expr".
    pub default_expr: Option<String>,

    pub varchar_length: Option<i32>,

    /// Comment inserted via `COMMENT ON COLUMN my_table.my_comment '...'`, if present.
    pub comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForeignKeyReference {
    pub schema: Option<String>,
    pub table: String,
    pub column: String,
}

impl SchemaCacheItem for Column {
    type Item = Column;

    async fn load(pool: &sqlx::PgPool) -> Result<Vec<Self::Item>, sqlx::Error> {
        sqlx::query_file_as!(Column, "src/queries/columns.sql")
            .fetch_all(pool)
            .await
    }
}

#[cfg(test)]
mod tests {
    use pglt_test_utils::test_database::get_new_test_db;
    use sqlx::Executor;

    use crate::{columns::ColumnClassKind, SchemaCache};

    #[tokio::test]
    async fn loads_columns() {
        let test_db = get_new_test_db().await;

        let setup = r#"
            create table public.users (
                id serial primary key,
                name varchar(255) not null,
                is_vegetarian bool default false,
                middle_name varchar(255)
            );

            create schema real_estate;

            create table real_estate.addresses (
                user_id serial references users(id),
                postal_code smallint not null,
                street text,
                city text
            );

            create table real_estate.properties (
                id serial primary key,
                owner_id int references users(id),
                square_meters smallint not null
            );

            comment on column real_estate.properties.owner_id is 'users might own many houses';
        "#;

        test_db
            .execute(setup)
            .await
            .expect("Failed to setup test database");

        let cache = SchemaCache::load(&test_db)
            .await
            .expect("Failed to load Schema Cache");

        let public_schema_columns = cache
            .columns
            .iter()
            .filter(|c| c.schema_name.as_str() == "public")
            .count();

        assert_eq!(public_schema_columns, 4);

        let real_estate_schema_columns = cache
            .columns
            .iter()
            .filter(|c| c.schema_name.as_str() == "real_estate")
            .count();

        assert_eq!(real_estate_schema_columns, 7);

        let user_id_col = cache.find_col("id", "users", None).unwrap();
        assert_eq!(user_id_col.class_kind, ColumnClassKind::OrdinaryTable);
        assert_eq!(user_id_col.comment, None);
        assert_eq!(
            user_id_col.default_expr,
            Some("nextval('users_id_seq'::regclass)".into())
        );
        assert!(!user_id_col.is_nullable);
        assert!(user_id_col.is_primary_key);
        assert!(user_id_col.is_unique);
        assert_eq!(user_id_col.varchar_length, None);

        let user_name_col = cache.find_col("name", "users", None).unwrap();
        assert_eq!(user_name_col.class_kind, ColumnClassKind::OrdinaryTable);
        assert_eq!(user_name_col.comment, None);
        assert_eq!(user_name_col.default_expr, None);
        assert!(!user_name_col.is_nullable);
        assert!(!user_name_col.is_primary_key);
        assert!(!user_name_col.is_unique);
        assert_eq!(user_name_col.varchar_length, Some(255));

        let user_is_veg_col = cache.find_col("is_vegetarian", "users", None).unwrap();
        assert_eq!(user_is_veg_col.class_kind, ColumnClassKind::OrdinaryTable);
        assert_eq!(user_is_veg_col.comment, None);
        assert_eq!(user_is_veg_col.default_expr, Some("false".into()));
        assert!(user_is_veg_col.is_nullable);
        assert!(!user_is_veg_col.is_primary_key);
        assert!(!user_is_veg_col.is_unique);
        assert_eq!(user_is_veg_col.varchar_length, None);

        let user_middle_name_col = cache.find_col("middle_name", "users", None).unwrap();
        assert_eq!(
            user_middle_name_col.class_kind,
            ColumnClassKind::OrdinaryTable
        );
        assert_eq!(user_middle_name_col.comment, None);
        assert_eq!(user_middle_name_col.default_expr, None);
        assert!(user_middle_name_col.is_nullable);
        assert!(!user_middle_name_col.is_primary_key);
        assert!(!user_middle_name_col.is_unique);
        assert_eq!(user_middle_name_col.varchar_length, Some(255));

        let properties_owner_id_col = cache
            .find_col("owner_id", "properties", Some("real_estate"))
            .unwrap();
        assert_eq!(
            properties_owner_id_col.class_kind,
            ColumnClassKind::OrdinaryTable
        );
        assert_eq!(
            properties_owner_id_col.comment,
            Some("users might own many houses".into())
        );
        assert!(properties_owner_id_col.is_nullable);
        assert!(!properties_owner_id_col.is_primary_key);
        assert!(!properties_owner_id_col.is_unique);
        assert_eq!(properties_owner_id_col.varchar_length, None);
    }
}
