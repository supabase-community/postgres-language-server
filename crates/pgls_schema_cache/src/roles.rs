use crate::schema_cache::SchemaCacheItem;

#[derive(Debug, PartialEq, Eq)]
pub struct Role {
    pub name: String,
    pub is_super_user: bool,
    pub can_create_db: bool,
    pub can_login: bool,
    pub can_bypass_rls: bool,
    pub can_create_roles: bool,
    pub comment: Option<String>,
    pub member_of: Vec<String>,
    pub has_member: Vec<String>,
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
    use sqlx::PgPool;

    use sqlx::Executor;

    use crate::SchemaCache;

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn loads_roles(test_db: PgPool) {
        let cache = SchemaCache::load(&test_db)
            .await
            .expect("Failed to load Schema Cache");

        let roles = &cache.roles;

        let super_role = roles.iter().find(|r| r.name == "owner").unwrap();
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

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn loads_comments(test_db: PgPool) {
        let setup = r#"
            comment on role owner is 'test';
        "#;

        test_db.execute(setup).await.expect("Setup failed");

        let cache = SchemaCache::load(&test_db)
            .await
            .expect("Failed to load Schema Cache");

        let roles = &cache.roles;

        let super_role = roles.iter().find(|r| r.name == "owner").unwrap();

        assert_eq!(super_role.comment, Some("test".into()))
    }
}
