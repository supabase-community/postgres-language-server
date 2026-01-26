create function polyf(x anyelement) returns anyelement as $$
  select x + 1
$$ language sql;

select polyf(42) as int, polyf(4.5) as num;

select polyf(point(3,4));

drop function polyf(x anyelement);

create function polyf(x anyelement) returns anyarray as $$
  select array[x + 1, x + 2]
$$ language sql;

select polyf(42) as int, polyf(4.5) as num;

drop function polyf(x anyelement);

create function polyf(x anyarray) returns anyelement as $$
  select x[1]
$$ language sql;

select polyf(array[2,4]) as int, polyf(array[4.5, 7.7]) as num;

select polyf(stavalues1) from pg_statistic;

drop function polyf(x anyarray);

create function polyf(x anyarray) returns anyarray as $$
  select x
$$ language sql;

select polyf(array[2,4]) as int, polyf(array[4.5, 7.7]) as num;

select polyf(stavalues1) from pg_statistic;

drop function polyf(x anyarray);

create function polyf(x anyelement) returns anyrange as $$
  select array[x + 1, x + 2]
$$ language sql;

create function polyf(x anyrange) returns anyarray as $$
  select array[lower(x), upper(x)]
$$ language sql;

select polyf(int4range(42, 49)) as int, polyf(float8range(4.5, 7.8)) as num;

drop function polyf(x anyrange);

create function polyf(x anycompatible, y anycompatible) returns anycompatiblearray as $$
  select array[x, y]
$$ language sql;

select polyf(2, 4) as int, polyf(2, 4.5) as num;

drop function polyf(x anycompatible, y anycompatible);

create function polyf(x anycompatiblerange, y anycompatible, z anycompatible) returns anycompatiblearray as $$
  select array[lower(x), upper(x), y, z]
$$ language sql;

select polyf(int4range(42, 49), 11, 2::smallint) as int, polyf(float8range(4.5, 7.8), 7.8, 11::real) as num;

select polyf(int4range(42, 49), 11, 4.5) as fail;

drop function polyf(x anycompatiblerange, y anycompatible, z anycompatible);

create function polyf(x anycompatiblemultirange, y anycompatible, z anycompatible) returns anycompatiblearray as $$
  select array[lower(x), upper(x), y, z]
$$ language sql;

select polyf(multirange(int4range(42, 49)), 11, 2::smallint) as int, polyf(multirange(float8range(4.5, 7.8)), 7.8, 11::real) as num;

select polyf(multirange(int4range(42, 49)), 11, 4.5) as fail;

drop function polyf(x anycompatiblemultirange, y anycompatible, z anycompatible);

create function polyf(x anycompatible) returns anycompatiblerange as $$
  select array[x + 1, x + 2]
$$ language sql;

create function polyf(x anycompatiblerange, y anycompatiblearray) returns anycompatiblerange as $$
  select x
$$ language sql;

select polyf(int4range(42, 49), array[11]) as int, polyf(float8range(4.5, 7.8), array[7]) as num;

drop function polyf(x anycompatiblerange, y anycompatiblearray);

create function polyf(x anycompatible) returns anycompatiblemultirange as $$
  select array[x + 1, x + 2]
$$ language sql;

create function polyf(x anycompatiblemultirange, y anycompatiblearray) returns anycompatiblemultirange as $$
  select x
$$ language sql;

select polyf(multirange(int4range(42, 49)), array[11]) as int, polyf(multirange(float8range(4.5, 7.8)), array[7]) as num;

drop function polyf(x anycompatiblemultirange, y anycompatiblearray);

create function polyf(a anyelement, b anyarray,
                      c anycompatible, d anycompatible,
                      OUT x anyarray, OUT y anycompatiblearray)
as $$
  select a || b, array[c, d]
$$ language sql;

select x, pg_typeof(x), y, pg_typeof(y)
  from polyf(11, array[1, 2], 42, 34.5);

select x, pg_typeof(x), y, pg_typeof(y)
  from polyf(11, array[1, 2], point(1,2), point(3,4));

select x, pg_typeof(x), y, pg_typeof(y)
  from polyf(11, '{1,2}', point(1,2), '(3,4)');

select x, pg_typeof(x), y, pg_typeof(y)
  from polyf(11, array[1, 2.2], 42, 34.5);

drop function polyf(a anyelement, b anyarray,
                    c anycompatible, d anycompatible);

create function polyf(anyrange) returns anymultirange
as 'select multirange($1);' language sql;

select polyf(int4range(1,10));

select polyf(null);

drop function polyf(anyrange);

create function polyf(anymultirange) returns anyelement
as 'select lower($1);' language sql;

select polyf(int4multirange(int4range(1,10), int4range(20,30)));

select polyf(null);

drop function polyf(anymultirange);

create function polyf(anycompatiblerange) returns anycompatiblemultirange
as 'select multirange($1);' language sql;

select polyf(int4range(1,10));

select polyf(null);

drop function polyf(anycompatiblerange);

create function polyf(anymultirange) returns anyrange
as 'select range_merge($1);' language sql;

select polyf(int4multirange(int4range(1,10), int4range(20,30)));

select polyf(null);

drop function polyf(anymultirange);

create function polyf(anycompatiblemultirange) returns anycompatiblerange
as 'select range_merge($1);' language sql;

select polyf(int4multirange(int4range(1,10), int4range(20,30)));

select polyf(null);

drop function polyf(anycompatiblemultirange);

create function polyf(anycompatiblemultirange) returns anycompatible
as 'select lower($1);' language sql;

select polyf(int4multirange(int4range(1,10), int4range(20,30)));

select polyf(null);

drop function polyf(anycompatiblemultirange);

CREATE FUNCTION stfp(anyarray) RETURNS anyarray AS
'select $1' LANGUAGE SQL;

CREATE FUNCTION stfnp(int[]) RETURNS int[] AS
'select $1' LANGUAGE SQL;

CREATE FUNCTION tfp(anyarray,anyelement) RETURNS anyarray AS
'select $1 || $2' LANGUAGE SQL;

CREATE FUNCTION tfnp(int[],int) RETURNS int[] AS
'select $1 || $2' LANGUAGE SQL;

CREATE FUNCTION tf1p(anyarray,int) RETURNS anyarray AS
'select $1' LANGUAGE SQL;

CREATE FUNCTION tf2p(int[],anyelement) RETURNS int[] AS
'select $1' LANGUAGE SQL;

CREATE FUNCTION sum3(anyelement,anyelement,anyelement) returns anyelement AS
'select $1+$2+$3' language sql strict;

CREATE FUNCTION ffp(anyarray) RETURNS anyarray AS
'select $1' LANGUAGE SQL;

CREATE FUNCTION ffnp(int[]) returns int[] as
'select $1' LANGUAGE SQL;

CREATE AGGREGATE myaggp01a(*) (SFUNC = stfnp, STYPE = int4[],
  FINALFUNC = ffp, INITCOND = '{}');

CREATE AGGREGATE myaggp02a(*) (SFUNC = stfnp, STYPE = anyarray,
  FINALFUNC = ffp, INITCOND = '{}');

CREATE AGGREGATE myaggp03a(*) (SFUNC = stfp, STYPE = int4[],
  FINALFUNC = ffp, INITCOND = '{}');

CREATE AGGREGATE myaggp03b(*) (SFUNC = stfp, STYPE = int4[],
  INITCOND = '{}');

CREATE AGGREGATE myaggp04a(*) (SFUNC = stfp, STYPE = anyarray,
  FINALFUNC = ffp, INITCOND = '{}');

CREATE AGGREGATE myaggp04b(*) (SFUNC = stfp, STYPE = anyarray,
  INITCOND = '{}');

CREATE AGGREGATE myaggp05a(BASETYPE = int, SFUNC = tfnp, STYPE = int[],
  FINALFUNC = ffp, INITCOND = '{}');

CREATE AGGREGATE myaggp06a(BASETYPE = int, SFUNC = tf2p, STYPE = int[],
  FINALFUNC = ffp, INITCOND = '{}');

CREATE AGGREGATE myaggp07a(BASETYPE = anyelement, SFUNC = tfnp, STYPE = int[],
  FINALFUNC = ffp, INITCOND = '{}');

CREATE AGGREGATE myaggp08a(BASETYPE = anyelement, SFUNC = tf2p, STYPE = int[],
  FINALFUNC = ffp, INITCOND = '{}');

CREATE AGGREGATE myaggp09a(BASETYPE = int, SFUNC = tf1p, STYPE = int[],
  FINALFUNC = ffp, INITCOND = '{}');

CREATE AGGREGATE myaggp09b(BASETYPE = int, SFUNC = tf1p, STYPE = int[],
  INITCOND = '{}');

CREATE AGGREGATE myaggp10a(BASETYPE = int, SFUNC = tfp, STYPE = int[],
  FINALFUNC = ffp, INITCOND = '{}');

CREATE AGGREGATE myaggp10b(BASETYPE = int, SFUNC = tfp, STYPE = int[],
  INITCOND = '{}');

CREATE AGGREGATE myaggp11a(BASETYPE = anyelement, SFUNC = tf1p, STYPE = int[],
  FINALFUNC = ffp, INITCOND = '{}');

CREATE AGGREGATE myaggp11b(BASETYPE = anyelement, SFUNC = tf1p, STYPE = int[],
  INITCOND = '{}');

CREATE AGGREGATE myaggp12a(BASETYPE = anyelement, SFUNC = tfp, STYPE = int[],
  FINALFUNC = ffp, INITCOND = '{}');

CREATE AGGREGATE myaggp12b(BASETYPE = anyelement, SFUNC = tfp, STYPE = int[],
  INITCOND = '{}');

CREATE AGGREGATE myaggp13a(BASETYPE = int, SFUNC = tfnp, STYPE = anyarray,
  FINALFUNC = ffp, INITCOND = '{}');

CREATE AGGREGATE myaggp14a(BASETYPE = int, SFUNC = tf2p, STYPE = anyarray,
  FINALFUNC = ffp, INITCOND = '{}');

CREATE AGGREGATE myaggp15a(BASETYPE = anyelement, SFUNC = tfnp,
  STYPE = anyarray, FINALFUNC = ffp, INITCOND = '{}');

CREATE AGGREGATE myaggp16a(BASETYPE = anyelement, SFUNC = tf2p,
  STYPE = anyarray, FINALFUNC = ffp, INITCOND = '{}');

CREATE AGGREGATE myaggp17a(BASETYPE = int, SFUNC = tf1p, STYPE = anyarray,
  FINALFUNC = ffp, INITCOND = '{}');

CREATE AGGREGATE myaggp17b(BASETYPE = int, SFUNC = tf1p, STYPE = anyarray,
  INITCOND = '{}');

CREATE AGGREGATE myaggp18a(BASETYPE = int, SFUNC = tfp, STYPE = anyarray,
  FINALFUNC = ffp, INITCOND = '{}');

CREATE AGGREGATE myaggp18b(BASETYPE = int, SFUNC = tfp, STYPE = anyarray,
  INITCOND = '{}');

CREATE AGGREGATE myaggp19a(BASETYPE = anyelement, SFUNC = tf1p,
  STYPE = anyarray, FINALFUNC = ffp, INITCOND = '{}');

CREATE AGGREGATE myaggp19b(BASETYPE = anyelement, SFUNC = tf1p,
  STYPE = anyarray, INITCOND = '{}');

CREATE AGGREGATE myaggp20a(BASETYPE = anyelement, SFUNC = tfp,
  STYPE = anyarray, FINALFUNC = ffp, INITCOND = '{}');

CREATE AGGREGATE myaggp20b(BASETYPE = anyelement, SFUNC = tfp,
  STYPE = anyarray, INITCOND = '{}');

CREATE AGGREGATE myaggn01a(*) (SFUNC = stfnp, STYPE = int4[],
  FINALFUNC = ffnp, INITCOND = '{}');

CREATE AGGREGATE myaggn01b(*) (SFUNC = stfnp, STYPE = int4[],
  INITCOND = '{}');

CREATE AGGREGATE myaggn02a(*) (SFUNC = stfnp, STYPE = anyarray,
  FINALFUNC = ffnp, INITCOND = '{}');

CREATE AGGREGATE myaggn02b(*) (SFUNC = stfnp, STYPE = anyarray,
  INITCOND = '{}');

CREATE AGGREGATE myaggn03a(*) (SFUNC = stfp, STYPE = int4[],
  FINALFUNC = ffnp, INITCOND = '{}');

CREATE AGGREGATE myaggn04a(*) (SFUNC = stfp, STYPE = anyarray,
  FINALFUNC = ffnp, INITCOND = '{}');

CREATE AGGREGATE myaggn05a(BASETYPE = int, SFUNC = tfnp, STYPE = int[],
  FINALFUNC = ffnp, INITCOND = '{}');

CREATE AGGREGATE myaggn05b(BASETYPE = int, SFUNC = tfnp, STYPE = int[],
  INITCOND = '{}');

CREATE AGGREGATE myaggn06a(BASETYPE = int, SFUNC = tf2p, STYPE = int[],
  FINALFUNC = ffnp, INITCOND = '{}');

CREATE AGGREGATE myaggn06b(BASETYPE = int, SFUNC = tf2p, STYPE = int[],
  INITCOND = '{}');

CREATE AGGREGATE myaggn07a(BASETYPE = anyelement, SFUNC = tfnp, STYPE = int[],
  FINALFUNC = ffnp, INITCOND = '{}');

CREATE AGGREGATE myaggn07b(BASETYPE = anyelement, SFUNC = tfnp, STYPE = int[],
  INITCOND = '{}');

CREATE AGGREGATE myaggn08a(BASETYPE = anyelement, SFUNC = tf2p, STYPE = int[],
  FINALFUNC = ffnp, INITCOND = '{}');

CREATE AGGREGATE myaggn08b(BASETYPE = anyelement, SFUNC = tf2p, STYPE = int[],
  INITCOND = '{}');

CREATE AGGREGATE myaggn09a(BASETYPE = int, SFUNC = tf1p, STYPE = int[],
  FINALFUNC = ffnp, INITCOND = '{}');

CREATE AGGREGATE myaggn10a(BASETYPE = int, SFUNC = tfp, STYPE = int[],
  FINALFUNC = ffnp, INITCOND = '{}');

CREATE AGGREGATE myaggn11a(BASETYPE = anyelement, SFUNC = tf1p, STYPE = int[],
  FINALFUNC = ffnp, INITCOND = '{}');

CREATE AGGREGATE myaggn12a(BASETYPE = anyelement, SFUNC = tfp, STYPE = int[],
  FINALFUNC = ffnp, INITCOND = '{}');

CREATE AGGREGATE myaggn13a(BASETYPE = int, SFUNC = tfnp, STYPE = anyarray,
  FINALFUNC = ffnp, INITCOND = '{}');

CREATE AGGREGATE myaggn13b(BASETYPE = int, SFUNC = tfnp, STYPE = anyarray,
  INITCOND = '{}');

CREATE AGGREGATE myaggn14a(BASETYPE = int, SFUNC = tf2p, STYPE = anyarray,
  FINALFUNC = ffnp, INITCOND = '{}');

CREATE AGGREGATE myaggn14b(BASETYPE = int, SFUNC = tf2p, STYPE = anyarray,
  INITCOND = '{}');

CREATE AGGREGATE myaggn15a(BASETYPE = anyelement, SFUNC = tfnp,
  STYPE = anyarray, FINALFUNC = ffnp, INITCOND = '{}');

CREATE AGGREGATE myaggn15b(BASETYPE = anyelement, SFUNC = tfnp,
  STYPE = anyarray, INITCOND = '{}');

CREATE AGGREGATE myaggn16a(BASETYPE = anyelement, SFUNC = tf2p,
  STYPE = anyarray, FINALFUNC = ffnp, INITCOND = '{}');

CREATE AGGREGATE myaggn16b(BASETYPE = anyelement, SFUNC = tf2p,
  STYPE = anyarray, INITCOND = '{}');

CREATE AGGREGATE myaggn17a(BASETYPE = int, SFUNC = tf1p, STYPE = anyarray,
  FINALFUNC = ffnp, INITCOND = '{}');

CREATE AGGREGATE myaggn18a(BASETYPE = int, SFUNC = tfp, STYPE = anyarray,
  FINALFUNC = ffnp, INITCOND = '{}');

CREATE AGGREGATE myaggn19a(BASETYPE = anyelement, SFUNC = tf1p,
  STYPE = anyarray, FINALFUNC = ffnp, INITCOND = '{}');

CREATE AGGREGATE myaggn20a(BASETYPE = anyelement, SFUNC = tfp,
  STYPE = anyarray, FINALFUNC = ffnp, INITCOND = '{}');

CREATE AGGREGATE mysum2(anyelement,anyelement) (SFUNC = sum3,
  STYPE = anyelement, INITCOND = '0');

create temp table t(f1 int, f2 int[], f3 text);

insert into t values(1,array[1],'a');

insert into t values(1,array[11],'b');

insert into t values(1,array[111],'c');

insert into t values(2,array[2],'a');

insert into t values(2,array[22],'b');

insert into t values(2,array[222],'c');

insert into t values(3,array[3],'a');

insert into t values(3,array[3],'b');

select f3, myaggp01a(*) from t group by f3 order by f3;

select f3, myaggp03a(*) from t group by f3 order by f3;

select f3, myaggp03b(*) from t group by f3 order by f3;

select f3, myaggp05a(f1) from t group by f3 order by f3;

select f3, myaggp06a(f1) from t group by f3 order by f3;

select f3, myaggp08a(f1) from t group by f3 order by f3;

select f3, myaggp09a(f1) from t group by f3 order by f3;

select f3, myaggp09b(f1) from t group by f3 order by f3;

select f3, myaggp10a(f1) from t group by f3 order by f3;

select f3, myaggp10b(f1) from t group by f3 order by f3;

select f3, myaggp20a(f1) from t group by f3 order by f3;

select f3, myaggp20b(f1) from t group by f3 order by f3;

select f3, myaggn01a(*) from t group by f3 order by f3;

select f3, myaggn01b(*) from t group by f3 order by f3;

select f3, myaggn03a(*) from t group by f3 order by f3;

select f3, myaggn05a(f1) from t group by f3 order by f3;

select f3, myaggn05b(f1) from t group by f3 order by f3;

select f3, myaggn06a(f1) from t group by f3 order by f3;

select f3, myaggn06b(f1) from t group by f3 order by f3;

select f3, myaggn08a(f1) from t group by f3 order by f3;

select f3, myaggn08b(f1) from t group by f3 order by f3;

select f3, myaggn09a(f1) from t group by f3 order by f3;

select f3, myaggn10a(f1) from t group by f3 order by f3;

select mysum2(f1, f1 + 1) from t;

create function bleat(int) returns int as $$
begin
  raise notice 'bleat %', $1;
  return $1;
end$$ language plpgsql;

create function sql_if(bool, anyelement, anyelement) returns anyelement as $$
select case when $1 then $2 else $3 end $$ language sql;

select f1, sql_if(f1 > 0, bleat(f1), bleat(f1 + 1)) from int4_tbl;

select q2, sql_if(q2 > 0, q2, q2 + 1) from int8_tbl;

CREATE AGGREGATE array_larger_accum (anyarray)
(
    sfunc = array_larger,
    stype = anyarray,
    initcond = '{}'
);

SELECT array_larger_accum(i)
FROM (VALUES (ARRAY[1,2]), (ARRAY[3,4])) as t(i);

SELECT array_larger_accum(i)
FROM (VALUES (ARRAY[row(1,2),row(3,4)]), (ARRAY[row(5,6),row(7,8)])) as t(i);

create function add_group(grp anyarray, ad anyelement, size integer)
  returns anyarray
  as $$
begin
  if grp is null then
    return array[ad];
  end if;
  if array_upper(grp, 1) < size then
    return grp || ad;
  end if;
  return grp;
end;
$$
  language plpgsql immutable;

create aggregate build_group(anyelement, integer) (
  SFUNC = add_group,
  STYPE = anyarray
);

select build_group(q1,3) from int8_tbl;

create aggregate build_group(int8, integer) (
  SFUNC = add_group,
  STYPE = int2[]
);

create aggregate build_group(int8, integer) (
  SFUNC = add_group,
  STYPE = int8[]
);

create function first_el_transfn(anyarray, anyelement) returns anyarray as
'select $1 || $2' language sql immutable;

create function first_el(anyarray) returns anyelement as
'select $1[1]' language sql strict immutable;

create aggregate first_el_agg_f8(float8) (
  SFUNC = array_append,
  STYPE = float8[],
  FINALFUNC = first_el
);

create aggregate first_el_agg_any(anyelement) (
  SFUNC = first_el_transfn,
  STYPE = anyarray,
  FINALFUNC = first_el
);

select first_el_agg_f8(x::float8) from generate_series(1,10) x;

select first_el_agg_any(x) from generate_series(1,10) x;

select first_el_agg_f8(x::float8) over(order by x) from generate_series(1,10) x;

select first_el_agg_any(x) over(order by x) from generate_series(1,10) x;

select distinct array_ndims(histogram_bounds) from pg_stats
where histogram_bounds is not null;

select max(histogram_bounds) from pg_stats where tablename = 'pg_am';

select array_in('{1,2,3}','int4'::regtype,-1);

select * from array_in('{1,2,3}','int4'::regtype,-1);

select anyrange_in('[10,20)','int4range'::regtype,-1);

create function myleast(variadic anyarray) returns anyelement as $$
  select min($1[i]) from generate_subscripts($1,1) g(i)
$$ language sql immutable strict;

select myleast(10, 1, 20, 33);

select myleast(1.1, 0.22, 0.55);

select myleast('z'::text);

select myleast();

select myleast(variadic array[1,2,3,4,-1]);

select myleast(variadic array[1.1, -5.5]);

select myleast(variadic array[]::int[]);

create function concat(text, variadic anyarray) returns text as $$
  select array_to_string($2, $1);
$$ language sql immutable strict;

select concat('%', 1, 2, 3, 4, 5);

select concat('|', 'a'::text, 'b', 'c');

select concat('|', variadic array[1,2,33]);

select concat('|', variadic array[]::int[]);

drop function concat(text, anyarray);

create function formarray(anyelement, variadic anyarray) returns anyarray as $$
  select array_prepend($1, $2);
$$ language sql immutable strict;

select formarray(1,2,3,4,5);

select formarray(1.1, variadic array[1.2,55.5]);

select formarray(1.1, array[1.2,55.5]);

select formarray(1, 'x'::text);

select formarray(1, variadic array['x'::text]);

drop function formarray(anyelement, variadic anyarray);

select pg_typeof(null);

select pg_typeof(0);

select pg_typeof(0.0);

select pg_typeof(1+1 = 2);

select pg_typeof('x');

select pg_typeof('' || '');

select pg_typeof(pg_typeof(0));

select pg_typeof(array[1.2,55.5]);

select pg_typeof(myleast(10, 1, 20, 33));

create function dfunc(a int = 1, int = 2) returns int as $$
  select $1 + $2;
$$ language sql;

select dfunc();

select dfunc(10);

select dfunc(10, 20);

select dfunc(10, 20, 30);

drop function dfunc();

drop function dfunc(int);

drop function dfunc(int, int);

create function dfunc(a int = 1, b int) returns int as $$
  select $1 + $2;
$$ language sql;

create function dfunc(a int = 1, out sum int, b int = 2) as $$
  select $1 + $2;
$$ language sql;

select dfunc();

drop function dfunc(int, int);

create function dfunc(a int DEFAULT 1.0, int DEFAULT '-1') returns int as $$
  select $1 + $2;
$$ language sql;

select dfunc();

create function dfunc(a text DEFAULT 'Hello', b text DEFAULT 'World') returns text as $$
  select $1 || ', ' || $2;
$$ language sql;

select dfunc();

select dfunc('Hi');

select dfunc('Hi', 'City');

select dfunc(0);

select dfunc(10, 20);

drop function dfunc(int, int);

drop function dfunc(text, text);

create function dfunc(int = 1, int = 2) returns int as $$
  select 2;
$$ language sql;

create function dfunc(int = 1, int = 2, int = 3, int = 4) returns int as $$
  select 4;
$$ language sql;

select dfunc();

select dfunc(1);

select dfunc(1, 2);

select dfunc(1, 2, 3);

select dfunc(1, 2, 3, 4);

drop function dfunc(int, int);

drop function dfunc(int, int, int, int);

create function dfunc(out int = 20) returns int as $$
  select 1;
$$ language sql;

create function dfunc(anyelement = 'World'::text) returns text as $$
  select 'Hello, ' || $1::text;
$$ language sql;

select dfunc();

select dfunc(0);

select dfunc(to_date('20081215','YYYYMMDD'));

select dfunc('City'::text);

drop function dfunc(anyelement);

create function dfunc(a variadic int[]) returns int as
$$ select array_upper($1, 1) $$ language sql;

select dfunc();

select dfunc(10);

select dfunc(10,20);

create or replace function dfunc(a variadic int[] default array[]::int[]) returns int as
$$ select array_upper($1, 1) $$ language sql;

select dfunc();

select dfunc(10);

select dfunc(10,20);

create or replace function dfunc(a variadic int[]) returns int as
$$ select array_upper($1, 1) $$ language sql;

drop function dfunc(a variadic int[]);

create function dfunc(int = 1, int = 2, int = 3) returns int as $$
  select 3;
$$ language sql;

create function dfunc(int = 1, int = 2) returns int as $$
  select 2;
$$ language sql;

create function dfunc(text) returns text as $$
  select $1;
$$ language sql;

select dfunc(1);

select dfunc('Hi');

drop function dfunc(int, int, int);

drop function dfunc(int, int);

drop function dfunc(text);

create function dfunc(a int, b int, c int = 0, d int = 0)
  returns table (a int, b int, c int, d int) as $$
  select $1, $2, $3, $4;
$$ language sql;

select (dfunc(10,20,30)).*;

select (dfunc(a := 10, b := 20, c := 30)).*;

select * from dfunc(a := 10, b := 20);

select * from dfunc(b := 10, a := 20);

select * from dfunc(0);

select * from dfunc(1,2);

select * from dfunc(1,2,c := 3);

select * from dfunc(1,2,d := 3);

select * from dfunc(x := 20, b := 10, x := 30);

select * from dfunc(10, b := 20, 30);

select * from dfunc(x := 10, b := 20, c := 30);

select * from dfunc(10, 10, a := 20);

select * from dfunc(1,c := 2,d := 3);

drop function dfunc(int, int, int, int);

create function xleast(x numeric, variadic arr numeric[])
  returns numeric as $$
  select least(x, min(arr[i])) from generate_subscripts(arr, 1) g(i);
$$ language sql;

select xleast(x => 1, variadic arr => array[2,3]);

select xleast(1, variadic arr => array[2,3]);

set search_path = pg_catalog;

select xleast(1, variadic arr => array[2,3]);

reset search_path;

select xleast(foo => 1, variadic arr => array[2,3]);

select xleast(x => 1, variadic array[2,3]);

select xleast(1, variadic x => array[2,3]);

select xleast(arr => array[1], variadic x => 3);

select xleast(arr => array[1], x => 3);

select xleast(arr => 1, variadic x => array[2,3]);

drop function xleast(x numeric, variadic arr numeric[]);

create function dfunc(a varchar, b numeric, c date = current_date)
  returns table (a varchar, b numeric, c date) as $$
  select $1, $2, $3;
$$ language sql;

select (dfunc('Hello World', 20, '2009-07-25'::date)).*;

select * from dfunc('Hello World', 20, '2009-07-25'::date);

select * from dfunc(c := '2009-07-25'::date, a := 'Hello World', b := 20);

select * from dfunc('Hello World', b := 20, c := '2009-07-25'::date);

select * from dfunc('Hello World', c := '2009-07-25'::date, b := 20);

select * from dfunc('Hello World', c := 20, b := '2009-07-25'::date);

drop function dfunc(varchar, numeric, date);

create function dfunc(a varchar = 'def a', out _a varchar, c numeric = NULL, out _c numeric)
returns record as $$
  select $1, $2;
$$ language sql;

select (dfunc()).*;

select * from dfunc();

select * from dfunc('Hello', 100);

select * from dfunc(a := 'Hello', c := 100);

select * from dfunc(c := 100, a := 'Hello');

select * from dfunc('Hello');

select * from dfunc('Hello', c := 100);

select * from dfunc(c := 100);

create or replace function dfunc(a varchar = 'def a', out _a varchar, x numeric = NULL, out _c numeric)
returns record as $$
  select $1, $2;
$$ language sql;

create or replace function dfunc(a varchar = 'def a', out _a varchar, numeric = NULL, out _c numeric)
returns record as $$
  select $1, $2;
$$ language sql;

drop function dfunc(varchar, numeric);

create function testpolym(a int, a int) returns int as $$ select 1;$$ language sql;

create function testpolym(int, out a int, out a int) returns int as $$ select 1;$$ language sql;

create function testpolym(out a int, inout a int) returns int as $$ select 1;$$ language sql;

create function testpolym(a int, inout a int) returns int as $$ select 1;$$ language sql;

create function testpolym(a int, out a int) returns int as $$ select $1;$$ language sql;

select testpolym(37);

drop function testpolym(int);

create function testpolym(a int) returns table(a int) as $$ select $1;$$ language sql;

select * from testpolym(37);

drop function testpolym(int);

create function dfunc(a anyelement, b anyelement = null, flag bool = true)
returns anyelement as $$
  select case when $3 then $1 else $2 end;
$$ language sql;

select dfunc(1,2);

select dfunc('a'::text, 'b');

select dfunc(a := 1, b := 2);

select dfunc(a := 'a'::text, b := 'b');

select dfunc(a := 'a'::text, b := 'b', flag := false);

select dfunc(b := 'b'::text, a := 'a');

select dfunc(a := 'a'::text, flag := true);

select dfunc(a := 'a'::text, flag := false);

select dfunc(b := 'b'::text, a := 'a', flag := true);

select dfunc('a'::text, 'b', false);

select dfunc('a'::text, 'b', flag := false);

select dfunc('a'::text, 'b', true);

select dfunc('a'::text, 'b', flag := true);

select dfunc(a => 1, b => 2);

select dfunc(a => 'a'::text, b => 'b');

select dfunc(a => 'a'::text, b => 'b', flag => false);

select dfunc(b => 'b'::text, a => 'a');

select dfunc(a => 'a'::text, flag => true);

select dfunc(a => 'a'::text, flag => false);

select dfunc(b => 'b'::text, a => 'a', flag => true);

select dfunc('a'::text, 'b', false);

select dfunc('a'::text, 'b', flag => false);

select dfunc('a'::text, 'b', true);

select dfunc('a'::text, 'b', flag => true);

select dfunc(a =>-1);

select dfunc(a =>+1);

select dfunc(a =>/**/1);

select dfunc(a =>--comment to be removed by psql
  1);

do $$
  declare r integer;
  begin
    select dfunc(a=>-- comment
      1) into r;
    raise info 'r = %', r;
  end;
$$;

CREATE VIEW dfview AS
   SELECT q1, q2,
     dfunc(q1,q2, flag := q1>q2) as c3,
     dfunc(q1, flag := q1<q2, b := q2) as c4
     FROM int8_tbl;

select * from dfview;

drop view dfview;

drop function dfunc(anyelement, anyelement, bool);

create function anyctest(anycompatible, anycompatible)
returns anycompatible as $$
  select greatest($1, $2)
$$ language sql;

select x, pg_typeof(x) from anyctest(11, 12) x;

select x, pg_typeof(x) from anyctest(11, 12.3) x;

select x, pg_typeof(x) from anyctest(11, point(1,2)) x;

select x, pg_typeof(x) from anyctest('11', '12.3') x;

drop function anyctest(anycompatible, anycompatible);

create function anyctest(anycompatible, anycompatible)
returns anycompatiblearray as $$
  select array[$1, $2]
$$ language sql;

select x, pg_typeof(x) from anyctest(11, 12) x;

select x, pg_typeof(x) from anyctest(11, 12.3) x;

select x, pg_typeof(x) from anyctest(11, array[1,2]) x;

drop function anyctest(anycompatible, anycompatible);

create function anyctest(anycompatible, anycompatiblearray)
returns anycompatiblearray as $$
  select array[$1] || $2
$$ language sql;

select x, pg_typeof(x) from anyctest(11, array[12]) x;

select x, pg_typeof(x) from anyctest(11, array[12.3]) x;

select x, pg_typeof(x) from anyctest(12.3, array[13]) x;

select x, pg_typeof(x) from anyctest(12.3, '{13,14.4}') x;

select x, pg_typeof(x) from anyctest(11, array[point(1,2)]) x;

select x, pg_typeof(x) from anyctest(11, 12) x;

drop function anyctest(anycompatible, anycompatiblearray);

create function anyctest(anycompatible, anycompatiblerange)
returns anycompatiblerange as $$
  select $2
$$ language sql;

select x, pg_typeof(x) from anyctest(11, int4range(4,7)) x;

select x, pg_typeof(x) from anyctest(11, numrange(4,7)) x;

select x, pg_typeof(x) from anyctest(11, 12) x;

select x, pg_typeof(x) from anyctest(11.2, int4range(4,7)) x;

select x, pg_typeof(x) from anyctest(11.2, '[4,7)') x;

drop function anyctest(anycompatible, anycompatiblerange);

create function anyctest(anycompatiblerange, anycompatiblerange)
returns anycompatible as $$
  select lower($1) + upper($2)
$$ language sql;

select x, pg_typeof(x) from anyctest(int4range(11,12), int4range(4,7)) x;

select x, pg_typeof(x) from anyctest(int4range(11,12), numrange(4,7)) x;

drop function anyctest(anycompatiblerange, anycompatiblerange);

create function anyctest(anycompatible)
returns anycompatiblerange as $$
  select $1
$$ language sql;

create function anyctest(anycompatible, anycompatiblemultirange)
returns anycompatiblemultirange as $$
  select $2
$$ language sql;

select x, pg_typeof(x) from anyctest(11, multirange(int4range(4,7))) x;

select x, pg_typeof(x) from anyctest(11, multirange(numrange(4,7))) x;

select x, pg_typeof(x) from anyctest(11, 12) x;

select x, pg_typeof(x) from anyctest(11.2, multirange(int4range(4,7))) x;

select x, pg_typeof(x) from anyctest(11.2, '{[4,7)}') x;

drop function anyctest(anycompatible, anycompatiblemultirange);

create function anyctest(anycompatiblemultirange, anycompatiblemultirange)
returns anycompatible as $$
  select lower($1) + upper($2)
$$ language sql;

select x, pg_typeof(x) from anyctest(multirange(int4range(11,12)), multirange(int4range(4,7))) x;

select x, pg_typeof(x) from anyctest(multirange(int4range(11,12)), multirange(numrange(4,7))) x;

drop function anyctest(anycompatiblemultirange, anycompatiblemultirange);

create function anyctest(anycompatible)
returns anycompatiblemultirange as $$
  select $1
$$ language sql;

create function anyctest(anycompatiblenonarray, anycompatiblenonarray)
returns anycompatiblearray as $$
  select array[$1, $2]
$$ language sql;

select x, pg_typeof(x) from anyctest(11, 12) x;

select x, pg_typeof(x) from anyctest(11, 12.3) x;

select x, pg_typeof(x) from anyctest(array[11], array[1,2]) x;

drop function anyctest(anycompatiblenonarray, anycompatiblenonarray);

create function anyctest(a anyelement, b anyarray,
                         c anycompatible, d anycompatible)
returns anycompatiblearray as $$
  select array[c, d]
$$ language sql;

select x, pg_typeof(x) from anyctest(11, array[1, 2], 42, 34.5) x;

select x, pg_typeof(x) from anyctest(11, array[1, 2], point(1,2), point(3,4)) x;

select x, pg_typeof(x) from anyctest(11, '{1,2}', point(1,2), '(3,4)') x;

select x, pg_typeof(x) from anyctest(11, array[1, 2.2], 42, 34.5) x;

drop function anyctest(a anyelement, b anyarray,
                       c anycompatible, d anycompatible);

create function anyctest(variadic anycompatiblearray)
returns anycompatiblearray as $$
  select $1
$$ language sql;

select x, pg_typeof(x) from anyctest(11, 12) x;

select x, pg_typeof(x) from anyctest(11, 12.2) x;

select x, pg_typeof(x) from anyctest(11, '12') x;

select x, pg_typeof(x) from anyctest(11, '12.2') x;

select x, pg_typeof(x) from anyctest(variadic array[11, 12]) x;

select x, pg_typeof(x) from anyctest(variadic array[11, 12.2]) x;

drop function anyctest(variadic anycompatiblearray);
