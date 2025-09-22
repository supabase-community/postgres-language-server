create function execute_text_query_index(query_sql text)
returns setof text
language plpgsql
as
$$
begin
  set enable_seqscan = off;
  set enable_bitmapscan = on;
  return query execute query_sql;
end;
$$;
