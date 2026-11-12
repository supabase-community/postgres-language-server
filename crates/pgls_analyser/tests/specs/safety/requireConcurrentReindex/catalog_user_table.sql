-- A table of the user whose name starts with pg_ isn't a system catalog, so it gets the fix. The
-- system catalog pg_class doesn't.
-- catalog: builtin
-- section: up transaction:false queries:separate
create table public.pg_audit_log (id int);
-- expect_lint/requireConcurrentReindex
reindex table pg_audit_log;
-- expect_lint/requireConcurrentReindex
reindex table pg_class;
