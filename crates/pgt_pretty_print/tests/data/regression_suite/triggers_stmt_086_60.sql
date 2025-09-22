create trigger oid_unchanged_trig after update on table_with_oids
	for each row
	when (new.tableoid = old.tableoid AND new.tableoid <> 0)
	execute procedure trigger_func('after_upd_oid_unchanged');
