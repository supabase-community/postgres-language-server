CREATE EVENT TRIGGER test_event_trigger_guc
	ON sql_drop
	WHEN TAG IN ('DROP POLICY') EXECUTE FUNCTION test_event_trigger_guc();
