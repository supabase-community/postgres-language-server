create type mood as enum ('happy', 'sad');
select null::mood;
-- expect_lint/unknownType
select null::missing_type;
