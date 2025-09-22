select (w).size = 'segsize' as ok
from (select pg_ls_waldir() w) ss where length((w).name) = 24 limit 1;
