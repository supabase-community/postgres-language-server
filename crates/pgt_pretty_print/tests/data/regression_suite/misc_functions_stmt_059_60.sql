select count(*) > 0 as ok from (select pg_ls_waldir()) ss;
