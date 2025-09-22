select null from pg_database group by grouping sets (()) for update;
