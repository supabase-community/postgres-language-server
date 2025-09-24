use crate::schema_cache::SchemaCacheItem;
use strum::{EnumIter, IntoEnumIterator};

#[derive(Debug, PartialEq, Eq)]
pub enum TriggerAffected {
    Row,
    Statement,
}

impl From<i16> for TriggerAffected {
    fn from(value: i16) -> Self {
        let is_row = 0b0000_0001;
        if value & is_row == is_row {
            Self::Row
        } else {
            Self::Statement
        }
    }
}

#[derive(Debug, PartialEq, Eq, EnumIter)]
pub enum TriggerEvent {
    Insert,
    Delete,
    Update,
    Truncate,
}

struct TriggerEvents(Vec<TriggerEvent>);

impl From<i16> for TriggerEvents {
    fn from(value: i16) -> Self {
        Self(
            TriggerEvent::iter()
                .filter(|variant| {
                    #[rustfmt::skip]
                    let mask = match variant {
                        TriggerEvent::Insert   => 0b0000_0100,
                        TriggerEvent::Delete   => 0b0000_1000,
                        TriggerEvent::Update   => 0b0001_0000,
                        TriggerEvent::Truncate => 0b0010_0000,
                    };
                    mask & value == mask
                })
                .collect(),
        )
    }
}

#[derive(Debug, PartialEq, Eq, EnumIter)]
pub enum TriggerTiming {
    Before,
    After,
    Instead,
}

impl TryFrom<i16> for TriggerTiming {
    type Error = ();
    fn try_from(value: i16) -> Result<Self, ()> {
        TriggerTiming::iter()
            .find(|variant| {
                match variant {
                    TriggerTiming::Instead => {
                        let mask = 0b0100_0000;
                        mask & value == mask
                    }
                    TriggerTiming::Before => {
                        let mask = 0b0000_0010;
                        mask & value == mask
                    }
                    TriggerTiming::After => {
                        let mask = 0b1011_1101;
                        // timing is "AFTER" if neither INSTEAD nor BEFORE bit are set.
                        mask | value == mask
                    }
                }
            })
            .ok_or(())
    }
}

pub struct TriggerQueried {
    name: String,
    table_name: String,
    table_schema: String,
    proc_name: String,
    proc_schema: String,
    details_bitmask: i16,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Trigger {
    pub name: String,
    pub table_name: String,
    pub table_schema: String,
    pub proc_name: String,
    pub proc_schema: String,
    pub affected: TriggerAffected,
    pub timing: TriggerTiming,
    pub events: Vec<TriggerEvent>,
}

impl From<TriggerQueried> for Trigger {
    fn from(value: TriggerQueried) -> Self {
        Self {
            name: value.name,
            table_name: value.table_name,
            proc_name: value.proc_name,
            proc_schema: value.proc_schema,
            table_schema: value.table_schema,
            affected: value.details_bitmask.into(),
            timing: value.details_bitmask.try_into().unwrap(),
            events: TriggerEvents::from(value.details_bitmask).0,
        }
    }
}

impl SchemaCacheItem for Trigger {
    type Item = Trigger;

    async fn load(pool: &sqlx::PgPool) -> Result<Vec<Self::Item>, sqlx::Error> {
        let results = sqlx::query_file_as!(TriggerQueried, "src/queries/triggers.sql")
            .fetch_all(pool)
            .await?;

        Ok(results.into_iter().map(|r| r.into()).collect())
    }
}

#[cfg(test)]
mod tests {

    use sqlx::{Executor, PgPool};

    use crate::{
        SchemaCache,
        triggers::{TriggerAffected, TriggerEvent, TriggerTiming},
    };

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn loads_triggers(test_db: PgPool) {
        let setup = r#"
                create table public.users (
                    id serial primary key,
                    name text
                );

                create or replace function public.log_user_insert()
                returns trigger as $$
                begin
                    -- dummy body
                    return new;
                end;
                $$ language plpgsql;

                create trigger trg_users_insert
                    before insert on public.users
                    for each row
                    execute function public.log_user_insert();

                create trigger trg_users_update
                    after update or insert on public.users
                    for each statement
                    execute function public.log_user_insert();

                create trigger trg_users_delete
                    before delete on public.users
                    for each row
                    execute function public.log_user_insert();
            "#;

        test_db
            .execute(setup)
            .await
            .expect("Failed to setup test database");

        let cache = SchemaCache::load(&test_db)
            .await
            .expect("Failed to load Schema Cache");

        let triggers: Vec<_> = cache
            .triggers
            .iter()
            .filter(|t| t.table_name == "users")
            .collect();
        assert_eq!(triggers.len(), 3);

        let insert_trigger = triggers
            .iter()
            .find(|t| t.name == "trg_users_insert")
            .unwrap();
        assert_eq!(insert_trigger.table_schema, "public");
        assert_eq!(insert_trigger.table_name, "users");
        assert_eq!(insert_trigger.timing, TriggerTiming::Before);
        assert_eq!(insert_trigger.affected, TriggerAffected::Row);
        assert!(insert_trigger.events.contains(&TriggerEvent::Insert));
        assert_eq!(insert_trigger.proc_name, "log_user_insert");

        let update_trigger = triggers
            .iter()
            .find(|t| t.name == "trg_users_update")
            .unwrap();
        assert_eq!(insert_trigger.table_schema, "public");
        assert_eq!(insert_trigger.table_name, "users");
        assert_eq!(update_trigger.timing, TriggerTiming::After);
        assert_eq!(update_trigger.affected, TriggerAffected::Statement);
        assert!(update_trigger.events.contains(&TriggerEvent::Update));
        assert!(update_trigger.events.contains(&TriggerEvent::Insert));
        assert_eq!(update_trigger.proc_name, "log_user_insert");

        let delete_trigger = triggers
            .iter()
            .find(|t| t.name == "trg_users_delete")
            .unwrap();
        assert_eq!(insert_trigger.table_schema, "public");
        assert_eq!(insert_trigger.table_name, "users");
        assert_eq!(delete_trigger.timing, TriggerTiming::Before);
        assert_eq!(delete_trigger.affected, TriggerAffected::Row);
        assert!(delete_trigger.events.contains(&TriggerEvent::Delete));
        assert_eq!(delete_trigger.proc_name, "log_user_insert");
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn loads_instead_and_truncate_triggers(test_db: PgPool) {
        let setup = r#"
        create table public.docs (
            id serial primary key,
            content text
        );

        create view public.docs_view as
        select * from public.docs;

        create or replace function public.docs_instead_of_update()
        returns trigger as $$
        begin
            -- dummy body
            return new;
        end;
        $$ language plpgsql;

        create trigger trg_docs_instead_update
            instead of update on public.docs_view
            for each row
            execute function public.docs_instead_of_update();

        create or replace function public.docs_truncate()
        returns trigger as $$
        begin
            -- dummy body
            return null;
        end;
        $$ language plpgsql;

        create trigger trg_docs_truncate
            after truncate on public.docs
            for each statement
            execute function public.docs_truncate();
    "#;

        test_db
            .execute(setup)
            .await
            .expect("Failed to setup test database");

        let cache = SchemaCache::load(&test_db)
            .await
            .expect("Failed to load Schema Cache");

        let triggers: Vec<_> = cache
            .triggers
            .iter()
            .filter(|t| t.table_name == "docs" || t.table_name == "docs_view")
            .collect();
        assert_eq!(triggers.len(), 2);

        let instead_trigger = triggers
            .iter()
            .find(|t| t.name == "trg_docs_instead_update")
            .unwrap();
        assert_eq!(instead_trigger.table_schema, "public");
        assert_eq!(instead_trigger.table_name, "docs_view");
        assert_eq!(instead_trigger.timing, TriggerTiming::Instead);
        assert_eq!(instead_trigger.affected, TriggerAffected::Row);
        assert!(instead_trigger.events.contains(&TriggerEvent::Update));
        assert_eq!(instead_trigger.proc_name, "docs_instead_of_update");

        let truncate_trigger = triggers
            .iter()
            .find(|t| t.name == "trg_docs_truncate")
            .unwrap();
        assert_eq!(truncate_trigger.table_schema, "public");
        assert_eq!(truncate_trigger.table_name, "docs");
        assert_eq!(truncate_trigger.timing, TriggerTiming::After);
        assert_eq!(truncate_trigger.affected, TriggerAffected::Statement);
        assert!(truncate_trigger.events.contains(&TriggerEvent::Truncate));
        assert_eq!(truncate_trigger.proc_name, "docs_truncate");
    }
}
