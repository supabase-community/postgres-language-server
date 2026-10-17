-- expect_lint/requireConcurrentReindex
reindex (verbose) table users;
-- REINDEX SYSTEM doesn't support CONCURRENTLY, so there is no fix
-- expect_lint/requireConcurrentReindex
reindex system postgres;
