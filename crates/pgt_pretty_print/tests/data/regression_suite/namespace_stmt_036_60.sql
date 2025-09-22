CREATE FUNCTION fn(INT) RETURNS INT IMMUTABLE LANGUAGE plpgsql AS $$
  BEGIN
    RAISE NOTICE 'current search_path: %', current_setting('search_path');
    RETURN $1;
  END;
$$;
