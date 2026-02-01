CREATE FUNCTION widget_in(cstring)
   RETURNS widget
   AS 'regresslib'
   LANGUAGE C STRICT IMMUTABLE;

CREATE FUNCTION widget_out(widget)
   RETURNS cstring
   AS 'regresslib'
   LANGUAGE C STRICT IMMUTABLE;

CREATE FUNCTION int44in(cstring)
   RETURNS city_budget
   AS 'regresslib'
   LANGUAGE C STRICT IMMUTABLE;

CREATE FUNCTION int44out(city_budget)
   RETURNS cstring
   AS 'regresslib'
   LANGUAGE C STRICT IMMUTABLE;

CREATE TYPE widget (
   internallength = 24,
   input = widget_in,
   output = widget_out,
   typmod_in = numerictypmodin,
   typmod_out = numerictypmodout,
   alignment = double
);

CREATE TYPE city_budget (
   internallength = 16,
   input = int44in,
   output = int44out,
   element = int4,
   category = 'x',   -- just to verify the system will take it
   preferred = true  -- ditto
);

CREATE TYPE shell;

CREATE TYPE shell;

DROP TYPE shell;

DROP TYPE shell;

CREATE TYPE myshell;

CREATE TYPE int42;

CREATE TYPE text_w_default;

CREATE FUNCTION int42_in(cstring)
   RETURNS int42
   AS 'int4in'
   LANGUAGE internal STRICT IMMUTABLE;

CREATE FUNCTION int42_out(int42)
   RETURNS cstring
   AS 'int4out'
   LANGUAGE internal STRICT IMMUTABLE;

CREATE FUNCTION text_w_default_in(cstring)
   RETURNS text_w_default
   AS 'textin'
   LANGUAGE internal STRICT IMMUTABLE;

CREATE FUNCTION text_w_default_out(text_w_default)
   RETURNS cstring
   AS 'textout'
   LANGUAGE internal STRICT IMMUTABLE;

CREATE TYPE int42 (
   internallength = 4,
   input = int42_in,
   output = int42_out,
   alignment = int4,
   default = 42,
   passedbyvalue
);

CREATE TYPE text_w_default (
   internallength = variable,
   input = text_w_default_in,
   output = text_w_default_out,
   alignment = int4,
   default = 'zippo'
);

CREATE TABLE default_test (f1 text_w_default, f2 int42);

INSERT INTO default_test DEFAULT VALUES;

SELECT * FROM default_test;

CREATE TYPE bogus_type;

CREATE TYPE bogus_type (
	"Internallength" = 4,
	"Input" = int42_in,
	"Output" = int42_out,
	"Alignment" = int4,
	"Default" = 42,
	"Passedbyvalue"
);

CREATE TYPE bogus_type (INPUT = array_in,
    OUTPUT = array_out,
    ELEMENT = int,
    INTERNALLENGTH = 32);

DROP TYPE bogus_type;

CREATE TYPE bogus_type (INPUT = array_in,
    OUTPUT = array_out,
    ELEMENT = int,
    INTERNALLENGTH = 32);

CREATE TYPE default_test_row AS (f1 text_w_default, f2 int42);

CREATE FUNCTION get_default_test() RETURNS SETOF default_test_row AS '
  SELECT * FROM default_test;
' LANGUAGE SQL;

SELECT * FROM get_default_test();

COMMENT ON TYPE bad IS 'bad comment';

COMMENT ON TYPE default_test_row IS 'good comment';

COMMENT ON TYPE default_test_row IS NULL;

COMMENT ON COLUMN default_test_row.nope IS 'bad comment';

COMMENT ON COLUMN default_test_row.f1 IS 'good comment';

COMMENT ON COLUMN default_test_row.f1 IS NULL;

CREATE TYPE text_w_default;

DROP TYPE default_test_row CASCADE;

DROP TABLE default_test;

CREATE TYPE base_type;

CREATE FUNCTION base_fn_in(cstring) RETURNS base_type AS 'boolin'
    LANGUAGE internal IMMUTABLE STRICT;

CREATE FUNCTION base_fn_out(base_type) RETURNS cstring AS 'boolout'
    LANGUAGE internal IMMUTABLE STRICT;

CREATE TYPE base_type(INPUT = base_fn_in, OUTPUT = base_fn_out);

DROP FUNCTION base_fn_in(cstring);

DROP FUNCTION base_fn_out(base_type);

DROP TYPE base_type;

DROP TYPE base_type CASCADE;

CREATE TEMP TABLE mytab (foo widget(42,13,7));

CREATE TEMP TABLE mytab (foo widget(42,13));

SELECT format_type(atttypid,atttypmod) FROM pg_attribute
WHERE attrelid = 'mytab'::regclass AND attnum > 0;

INSERT INTO mytab VALUES ('(1,2,3)'), ('(-44,5.5,12)');

TABLE mytab;

select format_type('varchar'::regtype, 42);

select format_type('bpchar'::regtype, null);

select format_type('bpchar'::regtype, -1);

SELECT pg_input_is_valid('(1,2,3)', 'widget');

SELECT pg_input_is_valid('(1,2)', 'widget');

SELECT pg_input_is_valid('{"(1,2,3)"}', 'widget[]');

SELECT pg_input_is_valid('{"(1,2)"}', 'widget[]');

SELECT pg_input_is_valid('("(1,2,3)")', 'mytab');

SELECT pg_input_is_valid('("(1,2)")', 'mytab');

CREATE FUNCTION pt_in_widget(point, widget)
   RETURNS bool
   AS 'regresslib'
   LANGUAGE C STRICT;

CREATE OPERATOR <% (
   leftarg = point,
   rightarg = widget,
   procedure = pt_in_widget,
   commutator = >% ,
   negator = >=%
);

SELECT point '(1,2)' <% widget '(0,0,3)' AS t,
       point '(1,2)' <% widget '(0,0,1)' AS f;

CREATE TABLE city (
	name		name,
	location 	box,
	budget 		city_budget
);

INSERT INTO city VALUES
('Podunk', '(1,2),(3,4)', '100,127,1000'),
('Gotham', '(1000,34),(1100,334)', '123456,127,-1000,6789');

TABLE city;

CREATE TYPE myvarchar;

CREATE FUNCTION myvarcharin(cstring, oid, integer) RETURNS myvarchar
LANGUAGE internal IMMUTABLE PARALLEL SAFE STRICT AS 'varcharin';

CREATE FUNCTION myvarcharout(myvarchar) RETURNS cstring
LANGUAGE internal IMMUTABLE PARALLEL SAFE STRICT AS 'varcharout';

CREATE FUNCTION myvarcharsend(myvarchar) RETURNS bytea
LANGUAGE internal STABLE PARALLEL SAFE STRICT AS 'varcharsend';

CREATE FUNCTION myvarcharrecv(internal, oid, integer) RETURNS myvarchar
LANGUAGE internal STABLE PARALLEL SAFE STRICT AS 'varcharrecv';

ALTER TYPE myvarchar SET (storage = extended);

CREATE TYPE myvarchar (
    input = myvarcharin,
    output = myvarcharout,
    alignment = integer,
    storage = main
);

CREATE DOMAIN myvarchardom AS myvarchar;

ALTER TYPE myvarchar SET (storage = plain);

ALTER TYPE myvarchar SET (storage = extended);

ALTER TYPE myvarchar SET (
    send = myvarcharsend,
    receive = myvarcharrecv,
    typmod_in = varchartypmodin,
    typmod_out = varchartypmodout,
    -- these are bogus, but it's safe as long as we don't use the type:
    analyze = ts_typanalyze,
    subscript = raw_array_subscript_handler
);

SELECT typinput, typoutput, typreceive, typsend, typmodin, typmodout,
       typanalyze, typsubscript, typstorage
FROM pg_type WHERE typname = 'myvarchar';

SELECT typinput, typoutput, typreceive, typsend, typmodin, typmodout,
       typanalyze, typsubscript, typstorage
FROM pg_type WHERE typname = '_myvarchar';

SELECT typinput, typoutput, typreceive, typsend, typmodin, typmodout,
       typanalyze, typsubscript, typstorage
FROM pg_type WHERE typname = 'myvarchardom';

SELECT typinput, typoutput, typreceive, typsend, typmodin, typmodout,
       typanalyze, typsubscript, typstorage
FROM pg_type WHERE typname = '_myvarchardom';

DROP FUNCTION myvarcharsend(myvarchar);

DROP TYPE myvarchar;

DROP TYPE myvarchar CASCADE;
