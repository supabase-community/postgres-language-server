CREATE EVENT TRIGGER my_event_trigger ON ddl_command_start
EXECUTE FUNCTION my_event_function();