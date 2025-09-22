create function execute_text_query_heap(query_sql text)
returns setof text
language plpgsql
as
$$
begin
  set enable_seqscan = on;
  set enable_bitmapscan = off;
  return query execute query_sql;
end;
$$;
