-- expect_lint/banAlterDatabaseOptions
alter database d with connection limit 5;
-- expect_lint/banAlterDatabaseOptions
alter database d reset search_path;
