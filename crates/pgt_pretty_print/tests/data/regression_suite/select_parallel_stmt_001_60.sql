select parallel_workers_to_launch as parallel_workers_to_launch_before,
       parallel_workers_launched as parallel_workers_launched_before
  from pg_stat_database
  where datname = current_database() ;
