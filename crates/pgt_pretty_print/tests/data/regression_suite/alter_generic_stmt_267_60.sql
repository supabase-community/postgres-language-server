SELECT nspname, stxname, rolname
  FROM pg_statistic_ext s, pg_namespace n, pg_authid a
 WHERE s.stxnamespace = n.oid AND s.stxowner = a.oid
   AND n.nspname in ('alt_nsp1', 'alt_nsp2')
 ORDER BY nspname, stxname;
