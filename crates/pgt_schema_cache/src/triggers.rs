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
                #[rustfmt::skip]
                    let mask = match variant {
                        TriggerTiming::Instead   => 0b0100_0000,
                        TriggerTiming::Before    => 0b0000_0010,
                        TriggerTiming::After     => 0b0000_0000, // before/after share same bit
                    };
                mask & value == mask
            })
            .ok_or(())
    }
}

pub struct TriggerQueried {
    name: String,
    table_name: String,
    schema_name: String,
    proc_name: String,
    details_bitmask: i16,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Trigger {
    name: String,
    table_name: String,
    schema_name: String,
    affected: TriggerAffected,
    timing: TriggerTiming,
    events: Vec<TriggerEvent>,
}

impl From<TriggerQueried> for Trigger {
    fn from(value: TriggerQueried) -> Self {
        Self {
            name: value.name,
            table_name: value.table_name,
            schema_name: value.schema_name,
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
    use pgt_test_utils::test_database::get_new_test_db;
    use sqlx::Executor;

    use crate::{
        SchemaCache,
        triggers::{TriggerAffected, TriggerEvent, TriggerTiming},
    };

    #[tokio::test]
    async fn loads_triggers() {
        let test_db = get_new_test_db().await;

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
                    after update on public.users
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
        assert_eq!(insert_trigger.timing, TriggerTiming::Before);
        assert_eq!(insert_trigger.affected, TriggerAffected::Row);
        assert!(insert_trigger.events.contains(&TriggerEvent::Insert));

        let update_trigger = triggers
            .iter()
            .find(|t| t.name == "trg_users_update")
            .unwrap();
        assert_eq!(update_trigger.timing, TriggerTiming::After);
        assert_eq!(update_trigger.affected, TriggerAffected::Statement);
        assert!(update_trigger.events.contains(&TriggerEvent::Update));

        let delete_trigger = triggers
            .iter()
            .find(|t| t.name == "trg_users_delete")
            .unwrap();
        assert_eq!(delete_trigger.timing, TriggerTiming::Before);
        assert_eq!(delete_trigger.affected, TriggerAffected::Row);
        assert!(delete_trigger.events.contains(&TriggerEvent::Delete));
    }
}
