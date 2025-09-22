create or replace function explain_analyze_inc_sort_nodes_verify_invariants(query text)
returns bool language plpgsql
as
$$
declare
  node jsonb;
  group_stats jsonb;
  group_key text;
  space_key text;
begin
  for node in select * from jsonb_array_elements(explain_analyze_inc_sort_nodes(query)) t loop
    for group_key in select unnest(array['Full-sort Groups', 'Pre-sorted Groups']::text[]) t loop
      group_stats := node->group_key;
      for space_key in select unnest(array['Sort Space Memory', 'Sort Space Disk']::text[]) t loop
        if (group_stats->space_key->'Peak Sort Space Used')::bigint < (group_stats->space_key->'Peak Sort Space Used')::bigint then
          raise exception '% has invalid max space < average space', group_key;
        end if;
      end loop;
    end loop;
  end loop;
  return true;
end;
$$;
