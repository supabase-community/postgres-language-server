CREATE ROLE regress_subscription_user LOGIN SUPERUSER;

CREATE ROLE regress_subscription_user2;

CREATE ROLE regress_subscription_user3 IN ROLE pg_create_subscription;

CREATE ROLE regress_subscription_user_dummy LOGIN NOSUPERUSER;

SET SESSION AUTHORIZATION 'regress_subscription_user';

BEGIN;

CREATE SUBSCRIPTION regress_testsub CONNECTION 'testconn' PUBLICATION testpub WITH (create_slot);

COMMIT;

CREATE SUBSCRIPTION regress_testsub CONNECTION 'testconn' PUBLICATION testpub;

CREATE SUBSCRIPTION regress_testsub CONNECTION 'dbname=regress_doesnotexist' PUBLICATION foo, testpub, foo WITH (connect = false);

CREATE SUBSCRIPTION regress_testsub CONNECTION 'dbname=regress_doesnotexist' PUBLICATION testpub WITH (connect = false);

COMMENT ON SUBSCRIPTION regress_testsub IS 'test subscription';

SELECT obj_description(s.oid, 'pg_subscription') FROM pg_subscription s;

SELECT subname, stats_reset IS NULL stats_reset_is_null FROM pg_stat_subscription_stats WHERE subname = 'regress_testsub';

SELECT pg_stat_reset_subscription_stats(oid) FROM pg_subscription WHERE subname = 'regress_testsub';

SELECT subname, stats_reset IS NULL stats_reset_is_null FROM pg_stat_subscription_stats WHERE subname = 'regress_testsub';

SELECT stats_reset as prev_stats_reset FROM pg_stat_subscription_stats WHERE subname = 'regress_testsub' ;

SELECT pg_stat_reset_subscription_stats(oid) FROM pg_subscription WHERE subname = 'regress_testsub';

SELECT 'prev_stats_reset' < stats_reset FROM pg_stat_subscription_stats WHERE subname = 'regress_testsub';

CREATE SUBSCRIPTION regress_testsub CONNECTION 'dbname=regress_doesnotexist' PUBLICATION testpub WITH (connect = false);

SET SESSION AUTHORIZATION 'regress_subscription_user2';

CREATE SUBSCRIPTION regress_testsub2 CONNECTION 'dbname=regress_doesnotexist' PUBLICATION foo WITH (connect = false);

SET SESSION AUTHORIZATION 'regress_subscription_user';

CREATE SUBSCRIPTION regress_testsub2 CONNECTION 'dbname=regress_doesnotexist' PUBLICATION testpub WITH (connect = false, copy_data = true);

CREATE SUBSCRIPTION regress_testsub2 CONNECTION 'dbname=regress_doesnotexist' PUBLICATION testpub WITH (connect = false, enabled = true);

CREATE SUBSCRIPTION regress_testsub2 CONNECTION 'dbname=regress_doesnotexist' PUBLICATION testpub WITH (connect = false, create_slot = true);

CREATE SUBSCRIPTION regress_testsub2 CONNECTION 'dbname=regress_doesnotexist' PUBLICATION testpub WITH (slot_name = NONE, enabled = true);

CREATE SUBSCRIPTION regress_testsub2 CONNECTION 'dbname=regress_doesnotexist' PUBLICATION testpub WITH (slot_name = NONE, enabled = false, create_slot = true);

CREATE SUBSCRIPTION regress_testsub2 CONNECTION 'dbname=regress_doesnotexist' PUBLICATION testpub WITH (slot_name = NONE);

CREATE SUBSCRIPTION regress_testsub2 CONNECTION 'dbname=regress_doesnotexist' PUBLICATION testpub WITH (slot_name = NONE, enabled = false);

CREATE SUBSCRIPTION regress_testsub2 CONNECTION 'dbname=regress_doesnotexist' PUBLICATION testpub WITH (slot_name = NONE, create_slot = false);

CREATE SUBSCRIPTION regress_testsub3 CONNECTION 'dbname=regress_doesnotexist' PUBLICATION testpub WITH (slot_name = NONE, connect = false);

ALTER SUBSCRIPTION regress_testsub3 ENABLE;

ALTER SUBSCRIPTION regress_testsub3 REFRESH PUBLICATION;

CREATE SUBSCRIPTION regress_testsub4 CONNECTION 'dbname=regress_doesnotexist' PUBLICATION testpub WITH (slot_name = NONE, connect = false, origin = foo);

CREATE SUBSCRIPTION regress_testsub4 CONNECTION 'dbname=regress_doesnotexist' PUBLICATION testpub WITH (slot_name = NONE, connect = false, origin = none);

ALTER SUBSCRIPTION regress_testsub4 SET (origin = any);

DROP SUBSCRIPTION regress_testsub3;

DROP SUBSCRIPTION regress_testsub4;

CREATE SUBSCRIPTION regress_testsub5 CONNECTION 'i_dont_exist=param' PUBLICATION testpub;

CREATE SUBSCRIPTION regress_testsub5 CONNECTION 'port=-1' PUBLICATION testpub;

ALTER SUBSCRIPTION regress_testsub CONNECTION 'foobar';

ALTER SUBSCRIPTION regress_testsub SET PUBLICATION testpub2, testpub3

ALTER SUBSCRIPTION regress_testsub CONNECTION 'dbname=regress_doesnotexist2';

ALTER SUBSCRIPTION regress_testsub SET (slot_name = 'newname');

ALTER SUBSCRIPTION regress_testsub SET (password_required = false);

ALTER SUBSCRIPTION regress_testsub SET (run_as_owner = true);

ALTER SUBSCRIPTION regress_testsub SET (password_required = true);

ALTER SUBSCRIPTION regress_testsub SET (run_as_owner = false);

ALTER SUBSCRIPTION regress_testsub SET (slot_name = '');

ALTER SUBSCRIPTION regress_doesnotexist CONNECTION 'dbname=regress_doesnotexist2';

ALTER SUBSCRIPTION regress_testsub SET (create_slot = false);

ALTER SUBSCRIPTION regress_testsub SKIP (lsn = '0/12345');

ALTER SUBSCRIPTION regress_testsub SKIP (lsn = NONE);

ALTER SUBSCRIPTION regress_testsub SKIP (lsn = '0/0');

BEGIN;

ALTER SUBSCRIPTION regress_testsub ENABLE;

ALTER SUBSCRIPTION regress_testsub DISABLE;

COMMIT;

SET ROLE regress_subscription_user_dummy;

ALTER SUBSCRIPTION regress_testsub RENAME TO regress_testsub_dummy;

RESET ROLE;

ALTER SUBSCRIPTION regress_testsub RENAME TO regress_testsub_foo;

ALTER SUBSCRIPTION regress_testsub_foo SET (synchronous_commit = local);

ALTER SUBSCRIPTION regress_testsub_foo SET (synchronous_commit = foobar);

ALTER SUBSCRIPTION regress_testsub_foo RENAME TO regress_testsub;

ALTER SUBSCRIPTION regress_testsub OWNER TO regress_subscription_user2;

BEGIN;

DROP SUBSCRIPTION regress_testsub;

COMMIT;

ALTER SUBSCRIPTION regress_testsub SET (slot_name = NONE);

BEGIN;

DROP SUBSCRIPTION regress_testsub;

COMMIT;

DROP SUBSCRIPTION IF EXISTS regress_testsub;

DROP SUBSCRIPTION regress_testsub;

CREATE SUBSCRIPTION regress_testsub CONNECTION 'dbname=regress_doesnotexist' PUBLICATION testpub WITH (connect = false, binary = foo);

CREATE SUBSCRIPTION regress_testsub CONNECTION 'dbname=regress_doesnotexist' PUBLICATION testpub WITH (connect = false, binary = true);

ALTER SUBSCRIPTION regress_testsub SET (binary = false);

ALTER SUBSCRIPTION regress_testsub SET (slot_name = NONE);

DROP SUBSCRIPTION regress_testsub;

CREATE SUBSCRIPTION regress_testsub CONNECTION 'dbname=regress_doesnotexist' PUBLICATION testpub WITH (connect = false, streaming = foo);

CREATE SUBSCRIPTION regress_testsub CONNECTION 'dbname=regress_doesnotexist' PUBLICATION testpub WITH (connect = false, streaming = true);

ALTER SUBSCRIPTION regress_testsub SET (streaming = parallel);

ALTER SUBSCRIPTION regress_testsub SET (streaming = false);

ALTER SUBSCRIPTION regress_testsub SET (slot_name = NONE);

ALTER SUBSCRIPTION regress_testsub ADD PUBLICATION testpub

ALTER SUBSCRIPTION regress_testsub ADD PUBLICATION testpub1, testpub1

ALTER SUBSCRIPTION regress_testsub ADD PUBLICATION testpub1, testpub2

ALTER SUBSCRIPTION regress_testsub ADD PUBLICATION testpub1, testpub2

ALTER SUBSCRIPTION regress_testsub DROP PUBLICATION testpub1, testpub1

ALTER SUBSCRIPTION regress_testsub DROP PUBLICATION testpub, testpub1, testpub2

ALTER SUBSCRIPTION regress_testsub DROP PUBLICATION testpub3

ALTER SUBSCRIPTION regress_testsub DROP PUBLICATION testpub1, testpub2

DROP SUBSCRIPTION regress_testsub;

CREATE SUBSCRIPTION regress_testsub CONNECTION 'dbname=regress_doesnotexist' PUBLICATION mypub
       WITH (connect = false, create_slot = false, copy_data = false);

ALTER SUBSCRIPTION regress_testsub ENABLE;

BEGIN;

ALTER SUBSCRIPTION regress_testsub SET PUBLICATION mypub

END;

BEGIN;

ALTER SUBSCRIPTION regress_testsub REFRESH PUBLICATION;

END;

CREATE FUNCTION func() RETURNS VOID AS
$$ ALTER SUBSCRIPTION regress_testsub SET PUBLICATION mypub WITH (refresh = true) $$ LANGUAGE SQL;

SELECT func();

ALTER SUBSCRIPTION regress_testsub DISABLE;

ALTER SUBSCRIPTION regress_testsub SET (slot_name = NONE);

DROP SUBSCRIPTION regress_testsub;

DROP FUNCTION func;

CREATE SUBSCRIPTION regress_testsub CONNECTION 'dbname=regress_doesnotexist' PUBLICATION testpub WITH (connect = false, two_phase = foo);

CREATE SUBSCRIPTION regress_testsub CONNECTION 'dbname=regress_doesnotexist' PUBLICATION testpub WITH (connect = false, two_phase = true);

ALTER SUBSCRIPTION regress_testsub SET (streaming = true);

ALTER SUBSCRIPTION regress_testsub SET (slot_name = NONE);

DROP SUBSCRIPTION regress_testsub;

CREATE SUBSCRIPTION regress_testsub CONNECTION 'dbname=regress_doesnotexist' PUBLICATION testpub WITH (connect = false, streaming = true, two_phase = true);

ALTER SUBSCRIPTION regress_testsub SET (slot_name = NONE);

DROP SUBSCRIPTION regress_testsub;

CREATE SUBSCRIPTION regress_testsub CONNECTION 'dbname=regress_doesnotexist' PUBLICATION testpub WITH (connect = false, disable_on_error = foo);

CREATE SUBSCRIPTION regress_testsub CONNECTION 'dbname=regress_doesnotexist' PUBLICATION testpub WITH (connect = false, disable_on_error = false);

ALTER SUBSCRIPTION regress_testsub SET (disable_on_error = true);

ALTER SUBSCRIPTION regress_testsub SET (slot_name = NONE);

DROP SUBSCRIPTION regress_testsub;

CREATE SUBSCRIPTION regress_testsub CONNECTION 'dbname=regress_doesnotexist' PUBLICATION testpub WITH (connect = false, retain_dead_tuples = foo);

CREATE SUBSCRIPTION regress_testsub CONNECTION 'dbname=regress_doesnotexist' PUBLICATION testpub WITH (connect = false, retain_dead_tuples = false);

ALTER SUBSCRIPTION regress_testsub SET (slot_name = NONE);

DROP SUBSCRIPTION regress_testsub;

CREATE SUBSCRIPTION regress_testsub CONNECTION 'dbname=regress_doesnotexist' PUBLICATION testpub WITH (connect = false, max_retention_duration = foo);

CREATE SUBSCRIPTION regress_testsub CONNECTION 'dbname=regress_doesnotexist' PUBLICATION testpub WITH (connect = false, max_retention_duration = 1000);

ALTER SUBSCRIPTION regress_testsub SET (max_retention_duration = 0);

ALTER SUBSCRIPTION regress_testsub SET (slot_name = NONE);

DROP SUBSCRIPTION regress_testsub;

SET SESSION AUTHORIZATION regress_subscription_user3;

CREATE SUBSCRIPTION regress_testsub CONNECTION 'dbname=regress_doesnotexist' PUBLICATION testpub WITH (connect = false);

RESET SESSION AUTHORIZATION;

GRANT CREATE ON DATABASE REGRESSION TO regress_subscription_user3;

SET SESSION AUTHORIZATION regress_subscription_user3;

CREATE SUBSCRIPTION regress_testsub CONNECTION 'dbname=regress_doesnotexist' PUBLICATION testpub WITH (connect = false);

RESET SESSION AUTHORIZATION;

GRANT CREATE ON DATABASE REGRESSION TO regress_subscription_user3;

SET SESSION AUTHORIZATION regress_subscription_user3;

CREATE SUBSCRIPTION regress_testsub CONNECTION 'dbname=regress_doesnotexist' PUBLICATION testpub WITH (connect = false, password_required = false);

RESET SESSION AUTHORIZATION;

GRANT CREATE ON DATABASE REGRESSION TO regress_subscription_user3;

SET SESSION AUTHORIZATION regress_subscription_user3;

CREATE SUBSCRIPTION regress_testsub CONNECTION 'dbname=regress_doesnotexist password=regress_fakepassword' PUBLICATION testpub WITH (connect = false);

ALTER SUBSCRIPTION regress_testsub OWNER TO regress_subscription_user;

ALTER SUBSCRIPTION regress_testsub RENAME TO regress_testsub2;

RESET SESSION AUTHORIZATION;

REVOKE pg_create_subscription FROM regress_subscription_user3;

SET SESSION AUTHORIZATION regress_subscription_user3;

ALTER SUBSCRIPTION regress_testsub2 RENAME TO regress_testsub;

RESET SESSION AUTHORIZATION;

REVOKE CREATE ON DATABASE REGRESSION FROM regress_subscription_user3;

SET SESSION AUTHORIZATION regress_subscription_user3;

ALTER SUBSCRIPTION regress_testsub RENAME TO regress_testsub2;

BEGIN;

ALTER SUBSCRIPTION regress_testsub SET (failover);

COMMIT;

ALTER SUBSCRIPTION regress_testsub SET (slot_name = NONE);

DROP SUBSCRIPTION regress_testsub;

RESET SESSION AUTHORIZATION;

DROP ROLE regress_subscription_user;

DROP ROLE regress_subscription_user2;

DROP ROLE regress_subscription_user3;

DROP ROLE regress_subscription_user_dummy;
