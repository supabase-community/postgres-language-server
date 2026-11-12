-- Before Postgres 12, ALTER TYPE ... ADD VALUE fails in a transaction block.
-- catalog: builtin
-- server_version_num: 110000
-- expect_lint/avoidAlterEnumAddValue
alter type my_enum add value 'new_value';
