-- expect_only_lint/safety/creatingEnum
-- Simple enum with two values
CREATE TYPE status AS ENUM ('active', 'inactive');
