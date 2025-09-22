create function some_tab_stmt_trig_func() returns trigger as
$$begin raise notice 'updating some_tab'; return NULL; end;$$
language plpgsql;
