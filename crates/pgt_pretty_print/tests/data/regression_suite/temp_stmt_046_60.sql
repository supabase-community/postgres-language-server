do $$
begin
  execute format($cmd$
    CREATE TEMP TABLE temptest (col text CHECK (col < %L)) ON COMMIT DROP
  $cmd$,
    (SELECT string_agg(g.i::text || ':' || random()::text, '|')
     FROM generate_series(1, 100) g(i)));
end$$;
