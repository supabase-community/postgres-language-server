CREATE OPERATOR ## (
   leftarg = path,
   rightarg = path,
   function = path_inter,
   commutator = ##
);

CREATE OPERATOR @#@ (
   rightarg = int8,		-- prefix
   procedure = factorial
);

CREATE OPERATOR #%# (
   leftarg = int8,		-- fail, postfix is no longer supported
   procedure = factorial
);

SELECT @#@ 24;

select @@##@@ 24;

set search_path = pg_catalog;

select @#@ 24;

reset search_path;

select @#@ 24.0;

COMMENT ON OPERATOR ###### (NONE, int4) IS 'bad prefix';

COMMENT ON OPERATOR ###### (int4, NONE) IS 'bad postfix';

COMMENT ON OPERATOR ###### (int4, int8) IS 'bad infix';

DROP OPERATOR ###### (NONE, int4);

DROP OPERATOR ###### (int4, NONE);

DROP OPERATOR ###### (int4, int8);

CREATE OPERATOR !=- (
   rightarg = int8,
   procedure = factorial
);

SELECT !=- 10;

SELECT 2 !=/**/ 1, 2 !=/**/ 2;

SELECT 2 !=-- comment to be removed by psql
  1;

DO $$ -- use DO to protect -- from psql
  declare r boolean;
  begin
    execute $e$ select 2 !=-- comment
      1 $e$ into r;
    raise info 'r = %', r;
  end;
$$;

SELECT true<>-1 BETWEEN 1 AND 1;

SELECT false<>/**/1 BETWEEN 1 AND 1;

SELECT false<=-1 BETWEEN 1 AND 1;

SELECT false>=-1 BETWEEN 1 AND 1;

SELECT 2<=/**/3, 3>=/**/2, 2<>/**/3;

SELECT 3<=/**/2, 2>=/**/3, 2<>/**/2;

BEGIN TRANSACTION;

CREATE ROLE regress_rol_op1;

CREATE SCHEMA schema_op1;

GRANT USAGE ON SCHEMA schema_op1 TO PUBLIC;

REVOKE USAGE ON SCHEMA schema_op1 FROM regress_rol_op1;

SET ROLE regress_rol_op1;

CREATE OPERATOR schema_op1.#*# (
   rightarg = int8,
   procedure = factorial
);

ROLLBACK;

BEGIN TRANSACTION;

CREATE OPERATOR #*# (
   leftarg = SETOF int8,
   procedure = factorial
);

ROLLBACK;

BEGIN TRANSACTION;

CREATE OPERATOR #*# (
   rightarg = SETOF int8,
   procedure = factorial
);

ROLLBACK;

BEGIN TRANSACTION;

CREATE OR REPLACE FUNCTION fn_op2(boolean, boolean)
RETURNS boolean AS $$
    SELECT NULL::BOOLEAN;
$$ LANGUAGE sql IMMUTABLE;

CREATE OPERATOR === (
    LEFTARG = boolean,
    RIGHTARG = boolean,
    PROCEDURE = fn_op2,
    COMMUTATOR = ===,
    NEGATOR = !==,
    RESTRICT = contsel,
    JOIN = contjoinsel,
    SORT1, SORT2, LTCMP, GTCMP, HASHES, MERGES
);

ROLLBACK;

CREATE OPERATOR #@%# (
   rightarg = int8,
   procedure = factorial,
   invalid_att = int8
);

CREATE OPERATOR #@%# (
   procedure = factorial
);

CREATE OPERATOR #@%# (
   rightarg = int8
);

BEGIN TRANSACTION;

CREATE ROLE regress_rol_op3;

CREATE TYPE type_op3 AS ENUM ('new', 'open', 'closed');

CREATE FUNCTION fn_op3(type_op3, int8)
RETURNS int8 AS $$
    SELECT NULL::int8;
$$ LANGUAGE sql IMMUTABLE;

REVOKE USAGE ON TYPE type_op3 FROM regress_rol_op3;

REVOKE USAGE ON TYPE type_op3 FROM PUBLIC;

SET ROLE regress_rol_op3;

CREATE OPERATOR #*# (
   leftarg = type_op3,
   rightarg = int8,
   procedure = fn_op3
);

ROLLBACK;

BEGIN TRANSACTION;

CREATE ROLE regress_rol_op4;

CREATE TYPE type_op4 AS ENUM ('new', 'open', 'closed');

CREATE FUNCTION fn_op4(int8, type_op4)
RETURNS int8 AS $$
    SELECT NULL::int8;
$$ LANGUAGE sql IMMUTABLE;

REVOKE USAGE ON TYPE type_op4 FROM regress_rol_op4;

REVOKE USAGE ON TYPE type_op4 FROM PUBLIC;

SET ROLE regress_rol_op4;

CREATE OPERATOR #*# (
   leftarg = int8,
   rightarg = type_op4,
   procedure = fn_op4
);

ROLLBACK;

BEGIN TRANSACTION;

CREATE ROLE regress_rol_op5;

CREATE TYPE type_op5 AS ENUM ('new', 'open', 'closed');

CREATE FUNCTION fn_op5(int8, int8)
RETURNS int8 AS $$
    SELECT NULL::int8;
$$ LANGUAGE sql IMMUTABLE;

REVOKE EXECUTE ON FUNCTION fn_op5(int8, int8) FROM regress_rol_op5;

REVOKE EXECUTE ON FUNCTION fn_op5(int8, int8) FROM PUBLIC;

SET ROLE regress_rol_op5;

CREATE OPERATOR #*# (
   leftarg = int8,
   rightarg = int8,
   procedure = fn_op5
);

ROLLBACK;

BEGIN TRANSACTION;

CREATE ROLE regress_rol_op6;

CREATE TYPE type_op6 AS ENUM ('new', 'open', 'closed');

CREATE FUNCTION fn_op6(int8, int8)
RETURNS type_op6 AS $$
    SELECT NULL::type_op6;
$$ LANGUAGE sql IMMUTABLE;

REVOKE USAGE ON TYPE type_op6 FROM regress_rol_op6;

REVOKE USAGE ON TYPE type_op6 FROM PUBLIC;

SET ROLE regress_rol_op6;

CREATE OPERATOR #*# (
   leftarg = int8,
   rightarg = int8,
   procedure = fn_op6
);

ROLLBACK;

BEGIN TRANSACTION;

CREATE OPERATOR === (
    leftarg = integer,
    rightarg = integer,
    procedure = int4eq,
    negator = ===
);

ROLLBACK;

BEGIN TRANSACTION;

CREATE OPERATOR === (
    leftarg = integer,
    rightarg = integer,
    procedure = int4eq,
    commutator = ===!!!
);

CREATE OPERATOR ===!!! (
    leftarg = integer,
    rightarg = integer,
    procedure = int4ne,
    negator = ===!!!
);

ROLLBACK;

CREATE OPERATOR === (
    leftarg = integer,
    rightarg = integer,
    procedure = int4eq,
    commutator = =
);

CREATE OPERATOR === (
    leftarg = integer,
    rightarg = integer,
    procedure = int4eq,
    negator = <>
);

CREATE OPERATOR ===
(
	"Leftarg" = box,
	"Rightarg" = box,
	"Procedure" = area_equal_function,
	"Commutator" = ===,
	"Negator" = !==,
	"Restrict" = area_restriction_function,
	"Join" = area_join_function,
	"Hashes",
	"Merges"
);
