select * from (select pg_ls_dir('.') a) a where a = 'base' limit 1;
