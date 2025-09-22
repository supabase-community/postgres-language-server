CREATE EVENT TRIGGER regress_event_trigger_report_end ON ddl_command_end
  EXECUTE PROCEDURE event_trigger_report_end();
