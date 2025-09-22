CREATE TRIGGER evtrg_nontemp_trig
  BEFORE INSERT ON evtrg_nontemp_table
  EXECUTE FUNCTION event_trigger_dummy_trigger();
