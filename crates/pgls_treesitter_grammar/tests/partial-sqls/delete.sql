delete from users u where u.id = 1;
delete from only users as u using posts p, comments c where p.user_id = u.id returning u.id, p.title;
delete from users where current of users_cursor;
