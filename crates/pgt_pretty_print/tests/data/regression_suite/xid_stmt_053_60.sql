select pg_current_xact_id() >= pg_snapshot_xmin(pg_current_snapshot());
