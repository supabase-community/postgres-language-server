DO $$
DECLARE curtid tid;
BEGIN
  LOOP
    INSERT INTO brin_summarize VALUES (1) RETURNING ctid INTO curtid;
    EXIT WHEN curtid > tid '(2, 0)';
  END LOOP;
END;
$$;
