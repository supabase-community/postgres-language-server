SELECT count(*) > 0 FROM pg_init_privs;

GRANT SELECT ON pg_proc TO CURRENT_USER;

GRANT SELECT (prosrc) ON pg_proc TO CURRENT_USER;

GRANT SELECT (rolname, rolsuper) ON pg_authid TO CURRENT_USER;
