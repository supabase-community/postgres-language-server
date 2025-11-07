create temp table copytest (
	style	text,
	test 	text,
	filler	int);

insert into copytest values('DOS',E'abc\r\ndef',1);

insert into copytest values('Unix',E'abc\ndef',2);

insert into copytest values('Mac',E'abc\rdef',3);

insert into copytest values(E'esc\\ape',E'a\\r\\\r\\\n\\nb',4);

copy copytest to 'filename' csv;

create temp table copytest2 (like copytest);

copy copytest2 from 'filename' csv;

select * from copytest except select * from copytest2;

truncate copytest2;

copy copytest to 'filename' csv quote '''' escape E'\\';

copy copytest2 from 'filename' csv quote '''' escape E'\\';

select * from copytest except select * from copytest2;

truncate copytest2;

copy copytest2(test) from 'filename' csv;

select test from copytest2 order by test collate "C";

truncate copytest2;

copy copytest2(test) from stdin;

select test from copytest2;

create temp table copytest3 (
	c1 int,
	"col with , comma" text,
	"col with "" quote"  int);

copy copytest3 from stdin csv header;

copy copytest3 to stdout csv header;

create temp table copytest4 (
	c1 int,
	"colname with tab: 	" text);

copy copytest4 from stdin (header);

copy copytest4 to stdout (header);

create temp table copytest5 (c1 int);

copy copytest5 from stdin (format csv, header 2);

truncate copytest5;

copy copytest5 from stdin (format csv, header 4);

select count(*) from copytest5;

truncate copytest5;

copy copytest5 from stdin (format csv, header 5);

select count(*) from copytest5;

create table parted_copytest (
	a int,
	b int,
	c text
) partition by list (b);

create table parted_copytest_a1 (c text, b int, a int);

create table parted_copytest_a2 (a int, c text, b int);

alter table parted_copytest attach partition parted_copytest_a1 for values in(1);

alter table parted_copytest attach partition parted_copytest_a2 for values in(2);

insert into parted_copytest select x,1,'One' from generate_series(1,1000) x;

insert into parted_copytest select x,2,'Two' from generate_series(1001,1010) x;

insert into parted_copytest select x,1,'One' from generate_series(1011,1020) x;

copy (select * from parted_copytest order by a) to 'filename';

truncate parted_copytest;

copy parted_copytest from 'filename';

begin;

truncate parted_copytest;

copy parted_copytest from 'filename' (freeze);

rollback;

select tableoid::regclass,count(*),sum(a) from parted_copytest
group by tableoid order by tableoid::regclass::name;

truncate parted_copytest;

create function part_ins_func() returns trigger language plpgsql as $$
begin
  return new;
end;
$$;

create trigger part_ins_trig
	before insert on parted_copytest_a2
	for each row
	execute procedure part_ins_func();

copy parted_copytest from 'filename';

select tableoid::regclass,count(*),sum(a) from parted_copytest
group by tableoid order by tableoid::regclass::name;

truncate table parted_copytest;

create index on parted_copytest (b);

drop trigger part_ins_trig on parted_copytest_a2;

copy parted_copytest from stdin;

select * from parted_copytest where b = 1;

select * from parted_copytest where b = 2;

drop table parted_copytest;

create table tab_progress_reporting (
	name text,
	age int4,
	location point,
	salary int4,
	manager name
);

create function notice_after_tab_progress_reporting() returns trigger AS
$$
declare report record;
begin
  -- The fields ignored here are the ones that may not remain
  -- consistent across multiple runs.  The sizes reported may differ
  -- across platforms, so just check if these are strictly positive.
  with progress_data as (
    select
       relid::regclass::text as relname,
       command,
       type,
       bytes_processed > 0 as has_bytes_processed,
       bytes_total > 0 as has_bytes_total,
       tuples_processed,
       tuples_excluded,
       tuples_skipped
      from pg_stat_progress_copy
      where pid = pg_backend_pid())
  select into report (to_jsonb(r)) as value
    from progress_data r;

  raise info 'progress: %', report.value::text;
  return new;
end;
$$ language plpgsql;

create trigger check_after_tab_progress_reporting
	after insert on tab_progress_reporting
	for each statement
	execute function notice_after_tab_progress_reporting();

copy tab_progress_reporting from stdin;

truncate tab_progress_reporting;

copy tab_progress_reporting from 'filename'
	where (salary < 2000);

copy tab_progress_reporting from stdin(on_error ignore);

drop trigger check_after_tab_progress_reporting on tab_progress_reporting;

drop function notice_after_tab_progress_reporting();

drop table tab_progress_reporting;

create table header_copytest (
	a int,
	b int,
	c text
);

alter table header_copytest drop column c;

alter table header_copytest add column c text;

copy header_copytest to stdout

copy header_copytest from stdin

copy header_copytest from stdin

SELECT * FROM header_copytest ORDER BY a;

alter table header_copytest drop column b;

copy header_copytest (c, a) from stdin

SELECT * FROM header_copytest ORDER BY a;

drop table header_copytest;

create temp table oversized_column_default (
    col1 varchar(5) DEFAULT 'more than 5 chars',
    col2 varchar(5));

copy oversized_column_default from stdin;

copy oversized_column_default (col2) from stdin;

copy oversized_column_default from stdin (default '');

drop table oversized_column_default;

CREATE TABLE parted_si (
  id int not null,
  data text not null,
  -- prevent use of bulk insert by having a volatile function
  rand float8 not null default random()
)
PARTITION BY LIST((id % 2));

CREATE TABLE parted_si_p_even PARTITION OF parted_si FOR VALUES IN (0);

CREATE TABLE parted_si_p_odd PARTITION OF parted_si FOR VALUES IN (1);

COPY parted_si(id, data) FROM 'filename';

SELECT tableoid::regclass, id % 2 = 0 is_even, count(*) from parted_si GROUP BY 1, 2 ORDER BY 1;

DROP TABLE parted_si;

begin;

create foreign data wrapper copytest_wrapper;

create server copytest_server foreign data wrapper copytest_wrapper;

create foreign table copytest_foreign_table (a int) server copytest_server;

copy copytest_foreign_table from stdin (freeze);

CREATE MATERIALIZED VIEW copytest_mv AS SELECT 1 AS id WITH NO DATA;

COPY copytest_mv(id) TO stdout

REFRESH MATERIALIZED VIEW copytest_mv;

COPY copytest_mv(id) TO stdout

DROP MATERIALIZED VIEW copytest_mv;
