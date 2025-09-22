SELECT pid AS checkpointer_pid FROM pg_stat_activity
  WHERE backend_type = 'checkpointer' ;
