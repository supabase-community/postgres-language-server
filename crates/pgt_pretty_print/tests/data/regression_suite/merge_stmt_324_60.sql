DO $$
DECLARE
  result integer;
BEGIN
MERGE INTO pa_target t
  USING pa_source s
  ON t.tid = s.sid
  WHEN MATCHED THEN
    UPDATE SET balance = balance + delta, val = val || ' updated by merge'
  WHEN NOT MATCHED THEN
    INSERT VALUES (sid, delta, 'inserted by merge')
  WHEN NOT MATCHED BY SOURCE THEN
    UPDATE SET val = val || ' not matched by source';
GET DIAGNOSTICS result := ROW_COUNT;
RAISE NOTICE 'ROW_COUNT = %', result;
END;
$$;
