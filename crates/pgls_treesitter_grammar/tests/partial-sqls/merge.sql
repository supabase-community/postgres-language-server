merge into users u using posts p on p.user_id = u.id when matched then delete;
merge into users using posts on posts.user_id = users.id when matched and posts.title <> users.name then update set name = posts.title when not matched then insert (id, name) values (posts.user_id, posts.title);
merge into users u using posts p on p.user_id = u.id when not matched by source then update set name = 'x' when not matched by target then insert default values;
merge into users u using posts p on p.user_id = u.id when matched then do nothing returning merge_action(), u.id;
