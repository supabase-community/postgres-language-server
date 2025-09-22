CREATE TRIGGER a_temp_trig
  BEFORE INSERT ON a_temp_tbl
  EXECUTE FUNCTION event_trigger_dummy_trigger();
