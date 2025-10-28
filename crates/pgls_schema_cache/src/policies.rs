use crate::schema_cache::SchemaCacheItem;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyCommand {
    Select,
    Insert,
    Update,
    Delete,
    All,
}

impl From<&str> for PolicyCommand {
    fn from(value: &str) -> Self {
        match value {
            "SELECT" => PolicyCommand::Select,
            "INSERT" => PolicyCommand::Insert,
            "UPDATE" => PolicyCommand::Update,
            "DELETE" => PolicyCommand::Delete,
            "ALL" => PolicyCommand::All,
            _ => panic!("Invalid Policy Command {value}"),
        }
    }
}
impl From<String> for PolicyCommand {
    fn from(value: String) -> Self {
        PolicyCommand::from(value.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PolicyQueried {
    name: String,
    table_name: String,
    schema_name: String,
    is_permissive: String,
    command: String,
    role_names: Option<Vec<String>>,
    security_qualification: Option<String>,
    with_check: Option<String>,
}

impl From<PolicyQueried> for Policy {
    fn from(value: PolicyQueried) -> Self {
        Self {
            name: value.name,
            table_name: value.table_name,
            schema_name: value.schema_name,
            is_permissive: value.is_permissive == "PERMISSIVE",
            command: PolicyCommand::from(value.command),
            role_names: value.role_names.unwrap_or_default(),
            security_qualification: value.security_qualification,
            with_check: value.with_check,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Policy {
    pub name: String,
    pub table_name: String,
    pub schema_name: String,
    pub is_permissive: bool,
    pub command: PolicyCommand,
    pub role_names: Vec<String>,
    pub security_qualification: Option<String>,
    pub with_check: Option<String>,
}

impl SchemaCacheItem for Policy {
    type Item = Policy;

    async fn load(pool: &sqlx::PgPool) -> Result<Vec<Self::Item>, sqlx::Error> {
        let policies = sqlx::query_file_as!(PolicyQueried, "src/queries/policies.sql")
            .fetch_all(pool)
            .await?;

        Ok(policies.into_iter().map(Policy::from).collect())
    }
}

#[cfg(test)]
mod tests {

    use sqlx::{Executor, PgPool};

    use crate::{SchemaCache, policies::PolicyCommand};

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn loads_policies(test_db: PgPool) {
        let setup = r#"
            create table public.users (
                id serial primary key,
                name varchar(255) not null
            );

            -- multiple policies to test various commands
            create policy public_policy
                on public.users
                for select
                to public
                using (true);

            create policy public_policy_del
                on public.users
                for delete
                to public
                using (true);

            create policy public_policy_ins
                on public.users
                for insert
                to public
                with check (true);

            create policy owner_policy
                on public.users
                for all
                to owner
                with check (true);

            create schema real_estate;

            create table real_estate.properties (
                id serial primary key,
                owner_id int not null
            );

            create policy test_nologin_policy
                on real_estate.properties
                for update
                to test_nologin
                using (owner_id = current_user::int);
        "#;

        test_db
            .execute(setup)
            .await
            .expect("Failed to setup test database");

        let cache = SchemaCache::load(&test_db)
            .await
            .expect("Failed to load Schema Cache");

        let public_policies = cache
            .policies
            .iter()
            .filter(|p| p.schema_name == "public")
            .count();

        assert_eq!(public_policies, 4);

        let real_estate_policies = cache
            .policies
            .iter()
            .filter(|p| p.schema_name == "real_estate")
            .count();

        assert_eq!(real_estate_policies, 1);

        let public_policy = cache
            .policies
            .iter()
            .find(|p| p.name == "public_policy")
            .unwrap();
        assert_eq!(public_policy.table_name, "users");
        assert_eq!(public_policy.schema_name, "public");
        assert!(public_policy.is_permissive);
        assert_eq!(public_policy.command, PolicyCommand::Select);
        assert_eq!(public_policy.role_names, vec!["public"]);
        assert_eq!(public_policy.security_qualification, Some("true".into()));
        assert_eq!(public_policy.with_check, None);

        let owner_policy = cache
            .policies
            .iter()
            .find(|p| p.name == "owner_policy")
            .unwrap();
        assert_eq!(owner_policy.table_name, "users");
        assert_eq!(owner_policy.schema_name, "public");
        assert!(owner_policy.is_permissive);
        assert_eq!(owner_policy.command, PolicyCommand::All);
        assert_eq!(owner_policy.role_names, vec!["owner"]);
        assert_eq!(owner_policy.security_qualification, None);
        assert_eq!(owner_policy.with_check, Some("true".into()));

        let owner_policy = cache
            .policies
            .iter()
            .find(|p| p.name == "test_nologin_policy")
            .unwrap();
        assert_eq!(owner_policy.table_name, "properties");
        assert_eq!(owner_policy.schema_name, "real_estate");
        assert!(owner_policy.is_permissive);
        assert_eq!(owner_policy.command, PolicyCommand::Update);
        assert_eq!(owner_policy.role_names, vec!["test_nologin"]);
        assert_eq!(
            owner_policy.security_qualification,
            Some("(owner_id = (CURRENT_USER)::integer)".into())
        );
        assert_eq!(owner_policy.with_check, None);
    }
}
