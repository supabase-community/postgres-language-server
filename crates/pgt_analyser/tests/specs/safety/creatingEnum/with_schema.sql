-- expect_only_lint/safety/creatingEnum
-- Enum type with schema qualification
CREATE TYPE myschema.status AS ENUM ('active', 'inactive', 'pending');
