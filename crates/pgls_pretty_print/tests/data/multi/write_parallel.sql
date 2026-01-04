begin;

set parallel_setup_cost=0;

set parallel_tuple_cost=0;

set min_parallel_table_scan_size=0;

set max_parallel_workers_per_gather=4;

create table parallel_write as
    select length(stringu1) from tenk1 group by length(stringu1);

create table parallel_write as
    select length(stringu1) from tenk1 group by length(stringu1);

drop table parallel_write;

select length(stringu1) into parallel_write
    from tenk1 group by length(stringu1);

select length(stringu1) into parallel_write
    from tenk1 group by length(stringu1);

drop table parallel_write;

create materialized view parallel_mat_view as
    select length(stringu1) from tenk1 group by length(stringu1);

create materialized view parallel_mat_view as
    select length(stringu1) from tenk1 group by length(stringu1);

create unique index on parallel_mat_view(length);

refresh materialized view parallel_mat_view;

refresh materialized view concurrently parallel_mat_view;

drop materialized view parallel_mat_view;

prepare prep_stmt as select length(stringu1) from tenk1 group by length(stringu1);

create table parallel_write as execute prep_stmt;

create table parallel_write as execute prep_stmt;

drop table parallel_write;

rollback;
