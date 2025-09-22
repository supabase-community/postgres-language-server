create event trigger regress_event_trigger_end on ddl_command_end
   execute function test_event_trigger();
