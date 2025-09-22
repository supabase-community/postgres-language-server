DO $$
DECLARE r text[];
BEGIN
  r := parse_ident('Schemax.Tabley');
  RAISE NOTICE '%', format('%I.%I', r[1], r[2]);
  r := parse_ident('"SchemaX"."TableY"');
  RAISE NOTICE '%', format('%I.%I', r[1], r[2]);
END;
$$;
