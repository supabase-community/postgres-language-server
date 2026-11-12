create table public.users (id bigint primary key generated always as identity, email text not null unique, name varchar(255) default 'x', team_id int references teams (id) on delete cascade, created_at timestamptz not null default now(), check (id > 0), constraint users_email_key unique (email));
create unlogged table t (a int, b text collate "C") inherits (p, q);
create table t (a int) partition by range (a);
create table t partition of p for values in (1, 2);
create table t (like p);
create temp table t as select 1;
