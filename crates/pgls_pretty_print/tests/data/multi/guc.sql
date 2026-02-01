SHOW datestyle;

SET intervalstyle to 'asd';

SET vacuum_cost_delay TO 40;

SET datestyle = 'ISO, YMD';

SHOW vacuum_cost_delay;

SHOW datestyle;

SELECT '2006-08-13 12:34:56'::timestamptz;

SET LOCAL vacuum_cost_delay TO 50;

SHOW vacuum_cost_delay;

SET LOCAL datestyle = 'SQL';

SHOW datestyle;

SELECT '2006-08-13 12:34:56'::timestamptz;

BEGIN;

SET LOCAL vacuum_cost_delay TO 50;

SHOW vacuum_cost_delay;

SET LOCAL datestyle = 'SQL';

SHOW datestyle;

SELECT '2006-08-13 12:34:56'::timestamptz;

COMMIT;

SHOW vacuum_cost_delay;

SHOW datestyle;

SELECT '2006-08-13 12:34:56'::timestamptz;

BEGIN;

SET vacuum_cost_delay TO 60;

SHOW vacuum_cost_delay;

SET datestyle = 'German';

SHOW datestyle;

SELECT '2006-08-13 12:34:56'::timestamptz;

ROLLBACK;

SHOW vacuum_cost_delay;

SHOW datestyle;

SELECT '2006-08-13 12:34:56'::timestamptz;

BEGIN;

SET vacuum_cost_delay TO 70;

SET datestyle = 'MDY';

SHOW datestyle;

SELECT '2006-08-13 12:34:56'::timestamptz;

SAVEPOINT first_sp;

SET vacuum_cost_delay TO 80.1;

SHOW vacuum_cost_delay;

SET datestyle = 'German, DMY';

SHOW datestyle;

SELECT '2006-08-13 12:34:56'::timestamptz;

ROLLBACK TO first_sp;

SHOW datestyle;

SELECT '2006-08-13 12:34:56'::timestamptz;

SAVEPOINT second_sp;

SET vacuum_cost_delay TO '900us';

SET datestyle = 'SQL, YMD';

SHOW datestyle;

SELECT '2006-08-13 12:34:56'::timestamptz;

SAVEPOINT third_sp;

SET vacuum_cost_delay TO 100;

SHOW vacuum_cost_delay;

SET datestyle = 'Postgres, MDY';

SHOW datestyle;

SELECT '2006-08-13 12:34:56'::timestamptz;

ROLLBACK TO third_sp;

SHOW vacuum_cost_delay;

SHOW datestyle;

SELECT '2006-08-13 12:34:56'::timestamptz;

ROLLBACK TO second_sp;

SHOW vacuum_cost_delay;

SHOW datestyle;

SELECT '2006-08-13 12:34:56'::timestamptz;

ROLLBACK;

SHOW vacuum_cost_delay;

SHOW datestyle;

SELECT '2006-08-13 12:34:56'::timestamptz;

BEGIN;

SHOW vacuum_cost_delay;

SHOW datestyle;

SELECT '2006-08-13 12:34:56'::timestamptz;

SAVEPOINT sp;

SET LOCAL vacuum_cost_delay TO 30;

SHOW vacuum_cost_delay;

SET LOCAL datestyle = 'Postgres, MDY';

SHOW datestyle;

SELECT '2006-08-13 12:34:56'::timestamptz;

ROLLBACK TO sp;

SHOW vacuum_cost_delay;

SHOW datestyle;

SELECT '2006-08-13 12:34:56'::timestamptz;

ROLLBACK;

SHOW vacuum_cost_delay;

SHOW datestyle;

SELECT '2006-08-13 12:34:56'::timestamptz;

BEGIN;

SHOW vacuum_cost_delay;

SHOW datestyle;

SELECT '2006-08-13 12:34:56'::timestamptz;

SAVEPOINT sp;

SET LOCAL vacuum_cost_delay TO 30;

SHOW vacuum_cost_delay;

SET LOCAL datestyle = 'Postgres, MDY';

SHOW datestyle;

SELECT '2006-08-13 12:34:56'::timestamptz;

RELEASE SAVEPOINT sp;

SHOW vacuum_cost_delay;

SHOW datestyle;

SELECT '2006-08-13 12:34:56'::timestamptz;

ROLLBACK;

SHOW vacuum_cost_delay;

SHOW datestyle;

SELECT '2006-08-13 12:34:56'::timestamptz;

BEGIN;

SET vacuum_cost_delay TO 40;

SET LOCAL vacuum_cost_delay TO 50;

SHOW vacuum_cost_delay;

SET datestyle = 'ISO, DMY';

SET LOCAL datestyle = 'Postgres, MDY';

SHOW datestyle;

SELECT '2006-08-13 12:34:56'::timestamptz;

COMMIT;

SHOW vacuum_cost_delay;

SHOW datestyle;

SELECT '2006-08-13 12:34:56'::timestamptz;

SET datestyle = iso, ymd;

SHOW datestyle;

SELECT '2006-08-13 12:34:56'::timestamptz;

RESET datestyle;

SHOW datestyle;

SELECT '2006-08-13 12:34:56'::timestamptz;

SET seq_page_cost TO 'NaN';

SET vacuum_cost_delay TO '10s';

SET no_such_variable TO 42;

SHOW custom.my_guc;

SET custom.my_guc = 42;

SHOW custom.my_guc;

RESET custom.my_guc;

SHOW custom.my_guc;

SET custom.my.qualified.guc = 'foo';

SHOW custom.my.qualified.guc;

SET custom."bad-guc" = 42;

SHOW custom."bad-guc";

SET special."weird name" = 'foo';

SHOW special."weird name";

SET plpgsql.extra_foo_warnings = true;

LOAD 'plpgsql';

SET plpgsql.extra_foo_warnings = true;

SHOW plpgsql.extra_foo_warnings;

SELECT relname FROM pg_class WHERE relname = 'reset_test';

DISCARD TEMP;

SELECT relname FROM pg_class WHERE relname = 'reset_test';
