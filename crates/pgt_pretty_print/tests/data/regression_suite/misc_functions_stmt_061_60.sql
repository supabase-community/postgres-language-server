select count(*) > 0 as ok from (select * from pg_ls_waldir() limit 1) ss;
