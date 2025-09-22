CREATE OR REPLACE FUNCTION joinview_upd_trig_fn() RETURNS trigger
LANGUAGE plpgsql AS
$$
BEGIN
  RAISE NOTICE 'UPDATE: % -> %', old, new;
  UPDATE foo SET f1 = new.f1, f3 = new.f3, f4 = new.f4 * 10
    FROM joinme WHERE f2 = f2j AND f2 = old.f2
    RETURNING WITH (new AS n) new.f1, n.f4 INTO new.f1, new.f4;  -- now ok
  RETURN NEW;
END;
$$;
