create table users (id int8, name text);
create table imports (id int8, name text);
merge into users using imports on imports.id = users.id when matched and imports.name is null then do nothing when matched then delete returning merge_action();
merge into users u using imports as i on i.id = u.id when matched then delete when not matched then insert values (i.id, i.name) when not matched by source then delete;
merge into users using imports on imports.id = users.id when matched then delete returning (select merge_action());
-- expect_lint/invalidMerge
with recursive c as (select 1 as id) merge into users using c on c.id = users.id when matched then delete;
-- expect_lint/invalidMerge
merge into users using imports on imports.id = users.id when matched then delete when matched and imports.name is null then do nothing;
-- expect_lint/invalidMerge
merge into users using users on true when matched then delete;
-- expect_lint/invalidMerge
merge into users u using imports u on true when matched then delete;
-- expect_lint/invalidMerge
merge into users using (select 1 as id) s join imports i using (id) as users on true when matched then delete;
-- expect_lint/invalidMerge
merge into users using imports on true when not matched then do nothing when matched then delete when not matched and true then do nothing;
