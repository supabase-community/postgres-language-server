select 1;

select;

select * from nonesuch;

select nonesuch from pg_database;

select * from pg_database where nonesuch = pg_database.datname;

select * from pg_database where pg_database.datname = nonesuch;

select distinct on (foobar) * from pg_database;

select null from pg_database group by datname for update;

select null from pg_database group by grouping sets (()) for update;

delete from nonesuch;

drop table nonesuch;

alter table nonesuch rename to newnonesuch;

alter table nonesuch rename to stud_emp;

alter table stud_emp rename to student;

alter table stud_emp rename to stud_emp;

alter table nonesuchrel rename column nonesuchatt to newnonesuchatt;

alter table emp rename column nonesuchatt to newnonesuchatt;

alter table emp rename column salary to manager;

alter table emp rename column salary to ctid;

abort;

end;

create aggregate newavg2 (sfunc = int4pl,
			  basetype = int4,
			  stype = int4,
			  finalfunc = int2um,
			  initcond = '0');

create aggregate newcnt1 (sfunc = int4inc,
			  stype = int4,
			  initcond = '0');

drop index nonesuch;

drop aggregate newcnt (nonesuch);

drop aggregate nonesuch (int4);

drop aggregate newcnt (float4);

drop function nonesuch();

drop type nonesuch;

drop operator === (int4, int4);

drop operator = (nonesuch, int4);

drop operator = (int4, nonesuch);

drop rule nonesuch on noplace;

select 1/0;

select 1::int8/0;

select 1/0::int8;

select 1::int2/0;

select 1/0::int2;

select 1::numeric/0;

select 1/0::numeric;

select 1::float8/0;

select 1/0::float8;

select 1::float4/0;

select 1/0::float4;
