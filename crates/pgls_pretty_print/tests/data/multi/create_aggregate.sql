CREATE AGGREGATE newavg (
   sfunc = int4_avg_accum, basetype = int4, stype = _int8,
   finalfunc = int8_avg,
   initcond1 = '{0,0}'
);

COMMENT ON AGGREGATE newavg_wrong (int4) IS 'an agg comment';

COMMENT ON AGGREGATE newavg (int4) IS 'an agg comment';

COMMENT ON AGGREGATE newavg (int4) IS NULL;

CREATE AGGREGATE newsum (
   sfunc1 = int4pl, basetype = int4, stype1 = int4,
   initcond1 = '0'
);

CREATE AGGREGATE newcnt (*) (
   sfunc = int8inc, stype = int8,
   initcond = '0', parallel = safe
);

CREATE AGGREGATE oldcnt (
   sfunc = int8inc, basetype = 'ANY', stype = int8,
   initcond = '0'
);

CREATE AGGREGATE newcnt ("any") (
   sfunc = int8inc_any, stype = int8,
   initcond = '0'
);

COMMENT ON AGGREGATE nosuchagg (*) IS 'should fail';

COMMENT ON AGGREGATE newcnt (*) IS 'an agg(*) comment';

COMMENT ON AGGREGATE newcnt ("any") IS 'an agg(any) comment';

create function sum3(int8,int8,int8) returns int8 as
'select $1 + $2 + $3' language sql strict immutable;

create aggregate sum2(int8,int8) (
   sfunc = sum3, stype = int8,
   initcond = '0'
);

create type aggtype as (a integer, b integer, c text);

create function aggf_trans(aggtype[],integer,integer,text) returns aggtype[]
as 'select array_append($1,ROW($2,$3,$4)::aggtype)'
language sql strict immutable;

create function aggfns_trans(aggtype[],integer,integer,text) returns aggtype[]
as 'select array_append($1,ROW($2,$3,$4)::aggtype)'
language sql immutable;

create aggregate aggfstr(integer,integer,text) (
   sfunc = aggf_trans, stype = aggtype[],
   initcond = '{}'
);

create aggregate aggfns(integer,integer,text) (
   sfunc = aggfns_trans, stype = aggtype[], sspace = 10000,
   initcond = '{}'
);

create function least_accum(int8, int8) returns int8 language sql as
  'select least($1, $2)';

create aggregate least_agg(int4) (
  stype = int8, sfunc = least_accum
);

drop function least_accum(int8, int8);

create function least_accum(anycompatible, anycompatible)
returns anycompatible language sql as
  'select least($1, $2)';

create aggregate least_agg(int4) (
  stype = int8, sfunc = least_accum
);

create aggregate least_agg(int8) (
  stype = int8, sfunc = least_accum
);

drop function least_accum(anycompatible, anycompatible) cascade;

create function least_accum(anyelement, variadic anyarray)
returns anyelement language sql as
  'select least($1, min($2[i])) from generate_subscripts($2,1) g(i)';

create aggregate least_agg(variadic items anyarray) (
  stype = anyelement, sfunc = least_accum
);

create function cleast_accum(anycompatible, variadic anycompatiblearray)
returns anycompatible language sql as
  'select least($1, min($2[i])) from generate_subscripts($2,1) g(i)';

create aggregate cleast_agg(variadic items anycompatiblearray) (
  stype = anycompatible, sfunc = cleast_accum
);

create aggregate my_percentile_disc(float8 ORDER BY anyelement) (
  stype = internal,
  sfunc = ordered_set_transition,
  finalfunc = percentile_disc_final,
  finalfunc_extra = true,
  finalfunc_modify = read_write
);

create aggregate my_rank(VARIADIC "any" ORDER BY VARIADIC "any") (
  stype = internal,
  sfunc = ordered_set_transition_multi,
  finalfunc = rank_final,
  finalfunc_extra = true,
  hypothetical
);

alter aggregate my_percentile_disc(float8 ORDER BY anyelement)
  rename to test_percentile_disc;

alter aggregate my_rank(VARIADIC "any" ORDER BY VARIADIC "any")
  rename to test_rank;

CREATE AGGREGATE sumdouble (float8)
(
    stype = float8,
    sfunc = float8pl,
    mstype = float8,
    msfunc = float8pl,
    minvfunc = float8mi
);

CREATE AGGREGATE myavg (numeric)
(
	stype = internal,
	sfunc = numeric_avg_accum,
	serialfunc = numeric_avg_serialize
);

CREATE AGGREGATE myavg (numeric)
(
	stype = internal,
	sfunc = numeric_avg_accum,
	serialfunc = numeric_avg_deserialize,
	deserialfunc = numeric_avg_deserialize
);

CREATE AGGREGATE myavg (numeric)
(
	stype = internal,
	sfunc = numeric_avg_accum,
	serialfunc = numeric_avg_serialize,
	deserialfunc = numeric_avg_serialize
);

CREATE AGGREGATE myavg (numeric)
(
	stype = internal,
	sfunc = numeric_avg_accum,
	serialfunc = numeric_avg_serialize,
	deserialfunc = numeric_avg_deserialize,
	combinefunc = int4larger
);

CREATE AGGREGATE myavg (numeric)
(
	stype = internal,
	sfunc = numeric_avg_accum,
	finalfunc = numeric_avg,
	serialfunc = numeric_avg_serialize,
	deserialfunc = numeric_avg_deserialize,
	combinefunc = numeric_avg_combine,
	finalfunc_modify = shareable  -- just to test a non-default setting
);

SELECT aggfnoid, aggtransfn, aggcombinefn, aggtranstype::regtype,
       aggserialfn, aggdeserialfn, aggfinalmodify
FROM pg_aggregate
WHERE aggfnoid = 'myavg'::REGPROC;

DROP AGGREGATE myavg (numeric);

CREATE AGGREGATE myavg (numeric)
(
	stype = internal,
	sfunc = numeric_avg_accum,
	finalfunc = numeric_avg
);

CREATE OR REPLACE AGGREGATE myavg (numeric)
(
	stype = internal,
	sfunc = numeric_avg_accum,
	finalfunc = numeric_avg,
	serialfunc = numeric_avg_serialize,
	deserialfunc = numeric_avg_deserialize,
	combinefunc = numeric_avg_combine,
	finalfunc_modify = shareable  -- just to test a non-default setting
);

SELECT aggfnoid, aggtransfn, aggcombinefn, aggtranstype::regtype,
       aggserialfn, aggdeserialfn, aggfinalmodify
FROM pg_aggregate
WHERE aggfnoid = 'myavg'::REGPROC;

CREATE OR REPLACE AGGREGATE myavg (numeric)
(
	stype = numeric,
	sfunc = numeric_add
);

SELECT aggfnoid, aggtransfn, aggcombinefn, aggtranstype::regtype,
       aggserialfn, aggdeserialfn, aggfinalmodify
FROM pg_aggregate
WHERE aggfnoid = 'myavg'::REGPROC;

CREATE OR REPLACE AGGREGATE myavg (numeric)
(
	stype = numeric,
	sfunc = numeric_add,
	finalfunc = numeric_out
);

CREATE OR REPLACE AGGREGATE myavg (order by numeric)
(
	stype = numeric,
	sfunc = numeric_add
);

create function sum4(int8,int8,int8,int8) returns int8 as
'select $1 + $2 + $3 + $4' language sql strict immutable;

CREATE OR REPLACE AGGREGATE sum3 (int8,int8,int8)
(
	stype = int8,
	sfunc = sum4
);

drop function sum4(int8,int8,int8,int8);

DROP AGGREGATE myavg (numeric);

CREATE AGGREGATE mysum (int)
(
	stype = int,
	sfunc = int4pl,
	parallel = pear
);

CREATE FUNCTION float8mi_n(float8, float8) RETURNS float8 AS
$$ SELECT $1 - $2; $$
LANGUAGE SQL;

CREATE AGGREGATE invalidsumdouble (float8)
(
    stype = float8,
    sfunc = float8pl,
    mstype = float8,
    msfunc = float8pl,
    minvfunc = float8mi_n
);

CREATE FUNCTION float8mi_int(float8, float8) RETURNS int AS
$$ SELECT CAST($1 - $2 AS INT); $$
LANGUAGE SQL;

CREATE AGGREGATE wrongreturntype (float8)
(
    stype = float8,
    sfunc = float8pl,
    mstype = float8,
    msfunc = float8pl,
    minvfunc = float8mi_int
);

CREATE AGGREGATE case_agg ( -- old syntax
	"Sfunc1" = int4pl,
	"Basetype" = int4,
	"Stype1" = int4,
	"Initcond1" = '0',
	"Parallel" = safe
);

CREATE AGGREGATE case_agg(float8)
(
	"Stype" = internal,
	"Sfunc" = ordered_set_transition,
	"Finalfunc" = percentile_disc_final,
	"Finalfunc_extra" = true,
	"Finalfunc_modify" = read_write,
	"Parallel" = safe
);
