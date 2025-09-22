select pg_visible_in_snapshot(pg_current_xact_id(), pg_current_snapshot());
