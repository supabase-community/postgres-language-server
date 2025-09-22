DO $$
DECLARE
  result integer;
BEGIN
MERGE INTO target t
USING source AS s
ON t.tid = s.sid
WHEN MATCHED AND s.sid = 3 THEN UPDATE SET balance = t.balance + s.delta
WHEN MATCHED THEN DELETE
WHEN NOT MATCHED THEN INSERT VALUES (sid, delta);
IF FOUND THEN
  RAISE NOTICE 'Found';
ELSE
  RAISE NOTICE 'Not found';
END IF;
GET DIAGNOSTICS result := ROW_COUNT;
RAISE NOTICE 'ROW_COUNT = %', result;
END;
$$;
