SELECT getdatabaseencoding() <> 'UTF8' AS skip_test ;

SET client_encoding TO UTF8;

CREATE COLLATION regress_builtin_c (
  provider = builtin, locale = 'C');

SELECT LOWER(U&'\00C1' COLLATE regress_builtin_c) = U&'\00C1';

SELECT UPPER(U&'\00E1' COLLATE regress_builtin_c) = U&'\00E1';

SELECT U&'\00C1\00E1' !~ '[[:alpha:]]' COLLATE regress_builtin_c;

DROP COLLATION regress_builtin_c;

CREATE COLLATION regress_pg_c_utf8 (
  provider = builtin, locale = 'C_UTF8');

CREATE COLLATION regress_pg_c_utf8 (
  provider = builtin, locale = 'C.UTF8');

DROP COLLATION regress_pg_c_utf8;

CREATE COLLATION regress_pg_c_utf8 (
  provider = builtin, locale = 'C.UTF-8');

CREATE TABLE test_pg_c_utf8 (
  t TEXT COLLATE PG_C_UTF8
);

INSERT INTO test_pg_c_utf8 VALUES
  ('abc DEF 123abc'),
  ('Ã¡bc sÃs Ãss DÃF'),
  ('ÇxxÇ ÇxxÇ ÇxxÇ'),
  (U&'ÎÎ» 1a \FF11a'),
  ('ÈºÈºÈº'),
  ('â±¥â±¥â±¥'),
  ('â±¥Èº');

SELECT
    t, lower(t), initcap(t), upper(t),
    length(convert_to(t, 'UTF8')) AS t_bytes,
    length(convert_to(lower(t), 'UTF8')) AS lower_t_bytes,
    length(convert_to(initcap(t), 'UTF8')) AS initcap_t_bytes,
    length(convert_to(upper(t), 'UTF8')) AS upper_t_bytes
  FROM test_pg_c_utf8;

DROP TABLE test_pg_c_utf8;

SELECT lower('ÎÎ£' COLLATE PG_C_UTF8);

SELECT lower('ÎÍºÎ£Íº' COLLATE PG_C_UTF8);

SELECT lower('ÎÎÎ£Î' COLLATE PG_C_UTF8);

SELECT 'xyz' ~ '[[:alnum:]]' COLLATE PG_C_UTF8;

SELECT 'xyz' !~ '[[:upper:]]' COLLATE PG_C_UTF8;

SELECT '@' !~ '[[:alnum:]]' COLLATE PG_C_UTF8;

SELECT '=' ~ '[[:punct:]]' COLLATE PG_C_UTF8;

SELECT 'a8a' ~ '[[:digit:]]' COLLATE PG_C_UTF8;

SELECT 'àµ§' !~ '\d' COLLATE PG_C_UTF8;

SELECT 'xYz' ~* 'XyZ' COLLATE PG_C_UTF8;

SELECT 'xAb' ~* '[W-Y]' COLLATE PG_C_UTF8;

SELECT 'xAb' !~* '[c-d]' COLLATE PG_C_UTF8;

SELECT 'Î' ~* '[Î³-Î»]' COLLATE PG_C_UTF8;

SELECT 'Î´' ~* '[Î-Î]' COLLATE PG_C_UTF8;

select casefold('AbCd 123 #$% Ä±iIÄ° áº Ã ÇÇÇ Î£ÏÏ' collate PG_C_UTF8);

CREATE COLLATION regress_pg_unicode_fast (
  provider = builtin, locale = 'unicode');

CREATE COLLATION regress_pg_unicode_fast (
  provider = builtin, locale = 'PG_UNICODE_FAST');

CREATE TABLE test_pg_unicode_fast (
  t TEXT COLLATE PG_UNICODE_FAST
);

INSERT INTO test_pg_unicode_fast VALUES
  ('abc DEF 123abc'),
  ('Ã¡bc sÃs Ãss DÃF'),
  ('ÇxxÇ ÇxxÇ ÇxxÇ'),
  (U&'ÎÎ» 1a \FF11a'),
  ('ÈºÈºÈº'),
  ('â±¥â±¥â±¥'),
  ('â±¥Èº');

SELECT
    t, lower(t), initcap(t), upper(t),
    length(convert_to(t, 'UTF8')) AS t_bytes,
    length(convert_to(lower(t), 'UTF8')) AS lower_t_bytes,
    length(convert_to(initcap(t), 'UTF8')) AS initcap_t_bytes,
    length(convert_to(upper(t), 'UTF8')) AS upper_t_bytes
  FROM test_pg_unicode_fast;

DROP TABLE test_pg_unicode_fast;

SELECT lower('ÎÎ£' COLLATE PG_UNICODE_FAST);

SELECT lower('ÎÎ£0' COLLATE PG_UNICODE_FAST);

SELECT lower('ÎÍÎ£Í' COLLATE PG_UNICODE_FAST);

SELECT lower('ÎÍÎ£Í' COLLATE PG_UNICODE_FAST);

SELECT lower('Î£' COLLATE PG_UNICODE_FAST);

SELECT lower('0Î£' COLLATE PG_UNICODE_FAST);

SELECT lower('ÎÎ£Î' COLLATE PG_UNICODE_FAST);

SELECT lower('ÎÍÎ£ÍÎ' COLLATE PG_UNICODE_FAST);

SELECT lower('ÎÍÎ£ÍÎ' COLLATE PG_UNICODE_FAST);

SELECT 'xyz' ~ '[[:alnum:]]' COLLATE PG_UNICODE_FAST;

SELECT 'xyz' !~ '[[:upper:]]' COLLATE PG_UNICODE_FAST;

SELECT '@' !~ '[[:alnum:]]' COLLATE PG_UNICODE_FAST;

SELECT '=' !~ '[[:punct:]]' COLLATE PG_UNICODE_FAST;

SELECT 'a8a' ~ '[[:digit:]]' COLLATE PG_UNICODE_FAST;

SELECT 'àµ§' ~ '\d' COLLATE PG_UNICODE_FAST;

SELECT 'xYz' ~* 'XyZ' COLLATE PG_UNICODE_FAST;

SELECT 'xAb' ~* '[W-Y]' COLLATE PG_UNICODE_FAST;

SELECT 'xAb' !~* '[c-d]' COLLATE PG_UNICODE_FAST;

SELECT 'Î' ~* '[Î³-Î»]' COLLATE PG_UNICODE_FAST;

SELECT 'Î´' ~* '[Î-Î]' COLLATE PG_UNICODE_FAST;

select casefold('AbCd 123 #$% Ä±iIÄ° áº Ã ÇÇÇ Î£ÏÏ' collate PG_UNICODE_FAST);
