-- After a DO block, anything could exist, so nothing is reported.
-- expect_no_diagnostics
do $$ begin execute 'create table created_dynamically (id int8)'; end $$;
select * from created_dynamically;
