select parallel_workers_to_launch > 'parallel_workers_to_launch_before'  AS wrk_to_launch,
       parallel_workers_launched > 'parallel_workers_launched_before' AS wrk_launched
  from pg_stat_database
  where datname = current_database();
