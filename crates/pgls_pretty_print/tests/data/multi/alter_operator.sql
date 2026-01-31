CREATE FUNCTION alter_op_test_fn(boolean, boolean)
RETURNS boolean AS $$ SELECT NULL::BOOLEAN; $$ LANGUAGE sql IMMUTABLE;

CREATE FUNCTION customcontsel(internal, oid, internal, integer)
RETURNS float8 AS 'contsel' LANGUAGE internal STABLE STRICT;

CREATE OPERATOR === (
    LEFTARG = boolean,
    RIGHTARG = boolean,
    PROCEDURE = alter_op_test_fn,
    COMMUTATOR = ===,
    NEGATOR = !==,
    RESTRICT = customcontsel,
    JOIN = contjoinsel,
    HASHES, MERGES
);

SELECT pg_describe_object(refclassid,refobjid,refobjsubid) as ref, deptype
FROM pg_depend
WHERE classid = 'pg_operator'::regclass AND
      objid = '===(bool,bool)'::regoperator
ORDER BY 1;

ALTER OPERATOR === (boolean, boolean) SET (RESTRICT = NONE);

ALTER OPERATOR === (boolean, boolean) SET (JOIN = NONE);

SELECT oprrest, oprjoin FROM pg_operator WHERE oprname = '==='
  AND oprleft = 'boolean'::regtype AND oprright = 'boolean'::regtype;

SELECT pg_describe_object(refclassid,refobjid,refobjsubid) as ref, deptype
FROM pg_depend
WHERE classid = 'pg_operator'::regclass AND
      objid = '===(bool,bool)'::regoperator
ORDER BY 1;

ALTER OPERATOR === (boolean, boolean) SET (RESTRICT = contsel);

ALTER OPERATOR === (boolean, boolean) SET (JOIN = contjoinsel);

SELECT oprrest, oprjoin FROM pg_operator WHERE oprname = '==='
  AND oprleft = 'boolean'::regtype AND oprright = 'boolean'::regtype;

SELECT pg_describe_object(refclassid,refobjid,refobjsubid) as ref, deptype
FROM pg_depend
WHERE classid = 'pg_operator'::regclass AND
      objid = '===(bool,bool)'::regoperator
ORDER BY 1;

ALTER OPERATOR === (boolean, boolean) SET (RESTRICT = NONE, JOIN = NONE);

SELECT oprrest, oprjoin FROM pg_operator WHERE oprname = '==='
  AND oprleft = 'boolean'::regtype AND oprright = 'boolean'::regtype;

SELECT pg_describe_object(refclassid,refobjid,refobjsubid) as ref, deptype
FROM pg_depend
WHERE classid = 'pg_operator'::regclass AND
      objid = '===(bool,bool)'::regoperator
ORDER BY 1;

ALTER OPERATOR === (boolean, boolean) SET (RESTRICT = customcontsel, JOIN = contjoinsel);

SELECT oprrest, oprjoin FROM pg_operator WHERE oprname = '==='
  AND oprleft = 'boolean'::regtype AND oprright = 'boolean'::regtype;

SELECT pg_describe_object(refclassid,refobjid,refobjsubid) as ref, deptype
FROM pg_depend
WHERE classid = 'pg_operator'::regclass AND
      objid = '===(bool,bool)'::regoperator
ORDER BY 1;

ALTER OPERATOR === (boolean, boolean) SET (RESTRICT = non_existent_func);

ALTER OPERATOR === (boolean, boolean) SET (JOIN = non_existent_func);

ALTER OPERATOR & (bit, bit) SET ("Restrict" = _int_contsel, "Join" = _int_contjoinsel);

CREATE USER regress_alter_op_user;

SET SESSION AUTHORIZATION regress_alter_op_user;

ALTER OPERATOR === (boolean, boolean) SET (RESTRICT = NONE);

RESET SESSION AUTHORIZATION;

CREATE FUNCTION alter_op_test_fn_bool_real(boolean, real)
RETURNS boolean AS $$ SELECT NULL::BOOLEAN; $$ LANGUAGE sql IMMUTABLE;

CREATE FUNCTION alter_op_test_fn_real_bool(real, boolean)
RETURNS boolean AS $$ SELECT NULL::BOOLEAN; $$ LANGUAGE sql IMMUTABLE;

CREATE OPERATOR === (
    LEFTARG = boolean,
    RIGHTARG = real,
    PROCEDURE = alter_op_test_fn_bool_real
);

CREATE OPERATOR ==== (
    LEFTARG = real,
    RIGHTARG = boolean,
    PROCEDURE = alter_op_test_fn_real_bool
);

CREATE OPERATOR !==== (
    LEFTARG = boolean,
    RIGHTARG = real,
    PROCEDURE = alter_op_test_fn_bool_real
);

ALTER OPERATOR === (boolean, real) SET (MERGES = false);

ALTER OPERATOR === (boolean, real) SET (HASHES = false);

ALTER OPERATOR === (boolean, real) SET (MERGES);

ALTER OPERATOR === (boolean, real) SET (HASHES);

SELECT oprcanmerge, oprcanhash
FROM pg_operator WHERE oprname = '==='
  AND oprleft = 'boolean'::regtype AND oprright = 'real'::regtype;

ALTER OPERATOR === (boolean, real) SET (COMMUTATOR = ====);

SELECT op.oprname AS operator_name, com.oprname AS commutator_name,
  com.oprcode AS commutator_func
  FROM pg_operator op
  INNER JOIN pg_operator com ON (op.oid = com.oprcom AND op.oprcom = com.oid)
  WHERE op.oprname = '==='
  AND op.oprleft = 'boolean'::regtype AND op.oprright = 'real'::regtype;

ALTER OPERATOR === (boolean, real) SET (NEGATOR = ===);

ALTER OPERATOR === (boolean, real) SET (NEGATOR = !====);

SELECT op.oprname AS operator_name, neg.oprname AS negator_name,
  neg.oprcode AS negator_func
  FROM pg_operator op
  INNER JOIN pg_operator neg ON (op.oid = neg.oprnegate AND op.oprnegate = neg.oid)
  WHERE op.oprname = '==='
  AND op.oprleft = 'boolean'::regtype AND op.oprright = 'real'::regtype;

ALTER OPERATOR === (boolean, real) SET (NEGATOR = !====);

ALTER OPERATOR === (boolean, real) SET (COMMUTATOR = ====);

ALTER OPERATOR === (boolean, real) SET (MERGES);

ALTER OPERATOR === (boolean, real) SET (HASHES);

SELECT oprcanmerge, oprcanhash,
       pg_describe_object('pg_operator'::regclass, oprcom, 0) AS commutator,
       pg_describe_object('pg_operator'::regclass, oprnegate, 0) AS negator
  FROM pg_operator WHERE oprname = '==='
  AND oprleft = 'boolean'::regtype AND oprright = 'real'::regtype;

CREATE OPERATOR @= (
    LEFTARG = real,
    RIGHTARG = boolean,
    PROCEDURE = alter_op_test_fn_real_bool
);

CREATE OPERATOR @!= (
    LEFTARG = boolean,
    RIGHTARG = real,
    PROCEDURE = alter_op_test_fn_bool_real
);

ALTER OPERATOR === (boolean, real) SET (COMMUTATOR = @=);

ALTER OPERATOR === (boolean, real) SET (NEGATOR = @!=);

ALTER OPERATOR === (boolean, real) SET (MERGES = false);

ALTER OPERATOR === (boolean, real) SET (HASHES = false);

ALTER OPERATOR @=(real, boolean) SET (COMMUTATOR = ===);

ALTER OPERATOR @!=(boolean, real) SET (NEGATOR = ===);

SELECT oprcanmerge, oprcanhash,
       pg_describe_object('pg_operator'::regclass, oprcom, 0) AS commutator,
       pg_describe_object('pg_operator'::regclass, oprnegate, 0) AS negator
  FROM pg_operator WHERE oprname = '==='
  AND oprleft = 'boolean'::regtype AND oprright = 'real'::regtype;

DROP USER regress_alter_op_user;

DROP OPERATOR === (boolean, boolean);

DROP OPERATOR === (boolean, real);

DROP OPERATOR ==== (real, boolean);

DROP OPERATOR !==== (boolean, real);

DROP OPERATOR @= (real, boolean);

DROP OPERATOR @!= (boolean, real);

DROP FUNCTION customcontsel(internal, oid, internal, integer);

DROP FUNCTION alter_op_test_fn(boolean, boolean);

DROP FUNCTION alter_op_test_fn_bool_real(boolean, real);

DROP FUNCTION alter_op_test_fn_real_bool(real, boolean);
