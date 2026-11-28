-- expect_no_diagnostics
create view v as select 1;
create function f() returns int language sql as $$ select 1 $$;
create rule r as on insert to t do instead nothing;
create trigger tr before insert on t for each row execute function f();
