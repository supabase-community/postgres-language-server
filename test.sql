-- Final comprehensive test

-- Statement starters
sel * from users;
i into users values (1);
up users set x = 1;
de from users;
cr table foo (id int);
al table foo add column x int;
dr table foo;
se timezone to 'UTC';
g select on users to public;
w foo as (select 1) select * from foo;
sh all;
t users;
cop users (id) from stdin;
comm on table users is 'x';
res timezone;
rev select on users from public;
e select 1;
explain an select 1;
v users;
m into t using s on t.id = s.id when matched then delete;

-- Clause keywords
select * f users w id = 1 g by id h count(*) > 1 o by id l 10 of 5;

-- JOIN
select * from a j b on a.id = b.id;
select * from a left j b on a.id = b.id;

-- CREATE/ALTER/DROP subtypes
cr ta foo (id int);
cr v foo as select 1;
cr i foo on bar (id);
cr f foo() returns void as $$ $$ language sql;

-- INTO and VALUES
i i users v (1);
insert into users values (1);
