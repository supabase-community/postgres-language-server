CREATE TYPE casttesttype;

CREATE FUNCTION casttesttype_in(cstring)
   RETURNS casttesttype
   AS 'textin'
   LANGUAGE internal STRICT IMMUTABLE;

CREATE FUNCTION casttesttype_out(casttesttype)
   RETURNS cstring
   AS 'textout'
   LANGUAGE internal STRICT IMMUTABLE;

CREATE TYPE casttesttype (
   internallength = variable,
   input = casttesttype_in,
   output = casttesttype_out,
   alignment = int4
);

CREATE FUNCTION casttestfunc(casttesttype) RETURNS int4 LANGUAGE SQL AS
$$ SELECT 1; $$;

SELECT casttestfunc('foo'::text);

CREATE CAST (text AS casttesttype) WITHOUT FUNCTION;

SELECT casttestfunc('foo'::text);

SELECT casttestfunc('foo'::text::casttesttype);

DROP CAST (text AS casttesttype);

CREATE CAST (text AS casttesttype) WITHOUT FUNCTION AS IMPLICIT;

SELECT casttestfunc('foo'::text);

SELECT 1234::int4::casttesttype;

CREATE CAST (int4 AS casttesttype) WITH INOUT;

SELECT 1234::int4::casttesttype;

DROP CAST (int4 AS casttesttype);

CREATE FUNCTION int4_casttesttype(int4) RETURNS casttesttype LANGUAGE SQL AS
$$ SELECT ('foo'::text || $1::text)::casttesttype; $$;

CREATE CAST (int4 AS casttesttype) WITH FUNCTION int4_casttesttype(int4) AS IMPLICIT;

SELECT 1234::int4::casttesttype;

DROP FUNCTION int4_casttesttype(int4) CASCADE;

CREATE FUNCTION bar_int4_text(int4) RETURNS text LANGUAGE SQL AS
$$ SELECT ('bar'::text || $1::text); $$;

CREATE CAST (int4 AS casttesttype) WITH FUNCTION bar_int4_text(int4) AS IMPLICIT;

SELECT 1234::int4::casttesttype;

SELECT pg_describe_object(classid, objid, objsubid) as obj,
       pg_describe_object(refclassid, refobjid, refobjsubid) as objref,
       deptype
FROM pg_depend
WHERE classid = 'pg_cast'::regclass AND
      objid = (SELECT oid FROM pg_cast
               WHERE castsource = 'int4'::regtype
                 AND casttarget = 'casttesttype'::regtype)
ORDER BY refclassid;
