create function explain_query_json(query_sql text)
returns table (explain_line json)
language plpgsql as
$$
begin
  set enable_seqscan = off;
  set enable_bitmapscan = on;
  return query execute 'EXPLAIN (ANALYZE, FORMAT json) ' || query_sql;
end;
$$;
