SELECT
  CASE 'foo'::text
    WHEN 'foo' THEN ARRAY['a', 'b', 'c', 'd'] || enum_range(NULL::casetestenum)::text[]
    ELSE ARRAY['x', 'y']
    END;
