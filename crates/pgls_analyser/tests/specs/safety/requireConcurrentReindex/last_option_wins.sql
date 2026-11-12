-- expect_lint/requireConcurrentReindex
reindex (concurrently, concurrently false) index my_index;
reindex (concurrently false) index concurrently my_index;
