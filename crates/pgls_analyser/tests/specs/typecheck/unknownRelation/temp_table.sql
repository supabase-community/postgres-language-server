-- Temporary tables created earlier in the file are known (#624, #692).
-- expect_no_diagnostics
create temp table scratch (id int8, name text);
insert into scratch (id, name) values (1, 'a');
select * from scratch;
select * from pg_temp.scratch;
