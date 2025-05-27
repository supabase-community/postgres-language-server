use crate::schema_cache::SchemaCacheItem;

#[derive(Debug, PartialEq, Eq)]
pub struct Role {
    pub name: String,
    pub is_super_user: bool,
    pub can_create_db: bool,
    pub can_login: bool,
    pub can_bypass_rls: bool,
}

impl SchemaCacheItem for Role {
    type Item = Role;

    async fn load(pool: &sqlx::PgPool) -> Result<Vec<Self::Item>, sqlx::Error> {
        sqlx::query_file_as!(Role, "src/queries/roles.sql")
            .fetch_all(pool)
            .await
    }
}

#[cfg(test)]
mod tests {
    use crate::SchemaCache;
    use pgt_test_utils::test_database::get_new_test_db;
    use sqlx::Executor;

    #[tokio::test]
    async fn loads_roles() {
        let setup = r#"
            do $$
            begin
                if not exists (
                    select from pg_catalog.pg_roles
                    where rolname = 'test_super'
                ) then
                    create role test_super superuser createdb login bypassrls;
                end if;
                if not exists (
                    select from pg_catalog.pg_roles
                    where rolname = 'test_nologin'
                ) then
                    create role test_nologin;
                end if;
                if not exists (
                    select from pg_catalog.pg_roles
                    where rolname = 'test_login'
                ) then
                    create role test_login login;
                end if;
            end $$;
        "#;

        let test_db = get_new_test_db().await;

        test_db
            .execute(setup)
            .await
            .expect("Failed to setup test database");

        let cache = SchemaCache::load(&test_db)
            .await
            .expect("Failed to load Schema Cache");

        let roles = &cache.roles;

        let super_role = roles.iter().find(|r| r.name == "test_super").unwrap();
        assert!(super_role.is_super_user);
        assert!(super_role.can_create_db);
        assert!(super_role.can_login);
        assert!(super_role.can_bypass_rls);

        let nologin_role = roles.iter().find(|r| r.name == "test_nologin").unwrap();
        assert!(!nologin_role.is_super_user);
        assert!(!nologin_role.can_create_db);
        assert!(!nologin_role.can_login);
        assert!(!nologin_role.can_bypass_rls);

        let login_role = roles.iter().find(|r| r.name == "test_login").unwrap();
        assert!(!login_role.is_super_user);
        assert!(!login_role.can_create_db);
        assert!(login_role.can_login);
        assert!(!login_role.can_bypass_rls);
    }
}
