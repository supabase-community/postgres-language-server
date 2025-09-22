create event trigger regress_event_trigger
   on ddl_command_start
   execute procedure pg_backend_pid();
