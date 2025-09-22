CREATE FUNCTION wait_for_hot_stats() RETURNS void AS $$
DECLARE
  start_time timestamptz := clock_timestamp();
  updated bool;
BEGIN
  -- we don't want to wait forever; loop will exit after 30 seconds
  FOR i IN 1 .. 300 LOOP
    SELECT (pg_stat_get_tuples_hot_updated('brin_hot'::regclass::oid) > 0) INTO updated;
    EXIT WHEN updated;

    -- wait a little
    PERFORM pg_sleep_for('100 milliseconds');
    -- reset stats snapshot so we can test again
    PERFORM pg_stat_clear_snapshot();
  END LOOP;
  -- report time waited in postmaster log (where it won't change test output)
  RAISE log 'wait_for_hot_stats delayed % seconds',
    EXTRACT(epoch FROM clock_timestamp() - start_time);
END
$$ LANGUAGE plpgsql;
