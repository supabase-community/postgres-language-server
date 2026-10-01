create schema users_hidden;
create table users_hidden.users (id uuid, first_name text, last_name text, email text);
-- GitHub issue #431.
-- expect_lint/functionReturnTypeMismatch
create function test_error_42P13_type_column_mismatch(user_id uuid) returns users_hidden.users language sql as
$$
select u.id, u.first_name from users_hidden.users u where u.id = user_id;
$$;
create function test_column_return(user_id uuid) returns users_hidden.users language sql as
$$
select * from users_hidden.users u where u.id = user_id;
$$;
-- expect_lint/functionReturnTypeMismatch
create function names() returns table (first_name text, last_name text) language sql as
'select first_name, last_name, email from users_hidden.users';
-- expect_lint/functionReturnTypeMismatch
create function forget_users(out deleted uuid, out email text) language sql as
'delete from users_hidden.users';
create function remove_users() returns setof users_hidden.users language sql as
'delete from users_hidden.users returning *';
-- expect_lint/functionReturnTypeMismatch
create function json_return() returns jsonb language sql as $$ select '{}' $$;
create table typed_return (id uuid, value integer);
-- expect_lint/functionReturnTypeMismatch
create function composite_type_return() returns typed_return language sql as $$
select '00000000-0000-0000-0000-000000000001'::uuid, '{}';
$$;
