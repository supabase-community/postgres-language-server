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
    use pgt_test_utils::test_database::{RoleWithArgs, get_new_test_db};

    #[tokio::test]
    async fn loads_roles() {
        let mut test_db = get_new_test_db().await;

        test_db
            .setup_roles(vec![
                RoleWithArgs {
                    role: "test_super".into(),
                    args: vec![
                        "superuser".into(),
                        "createdb".into(),
                        "login".into(),
                        "bypassrls".into(),
                    ],
                },
                RoleWithArgs {
                    role: "test_nologin".into(),
                    args: vec![],
                },
                RoleWithArgs {
                    role: "test_login".into(),
                    args: vec!["login".into()],
                },
            ])
            .await
            .expect("Unable to set up roles.");

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
