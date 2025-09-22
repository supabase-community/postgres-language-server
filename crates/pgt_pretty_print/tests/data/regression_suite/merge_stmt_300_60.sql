CREATE FUNCTION merge_func() RETURNS integer LANGUAGE plpgsql AS $$
DECLARE
  result integer;
BEGIN
MERGE INTO pa_target t
  USING pa_source s
  ON t.tid = s.sid
  WHEN MATCHED THEN
    UPDATE SET tid = tid + 1, balance = balance + delta, val = val || ' updated by merge'
  WHEN NOT MATCHED THEN
    INSERT VALUES (sid, delta, 'inserted by merge')
  WHEN NOT MATCHED BY SOURCE THEN
    UPDATE SET tid = 1, val = val || ' not matched by source';
IF FOUND THEN
  GET DIAGNOSTICS result := ROW_COUNT;
END IF;
RETURN result;
END;
$$;
