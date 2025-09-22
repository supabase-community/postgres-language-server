create or replace function explain_analyze_inc_sort_nodes_without_memory(query text)
returns jsonb language plpgsql
as
$$
declare
  nodes jsonb := '[]'::jsonb;
  node jsonb;
  group_key text;
  space_key text;
begin
  for node in select * from jsonb_array_elements(explain_analyze_inc_sort_nodes(query)) t loop
    for group_key in select unnest(array['Full-sort Groups', 'Pre-sorted Groups']::text[]) t loop
      for space_key in select unnest(array['Sort Space Memory', 'Sort Space Disk']::text[]) t loop
        node := jsonb_set(node, array[group_key, space_key, 'Average Sort Space Used'], '"NN"', false);
        node := jsonb_set(node, array[group_key, space_key, 'Peak Sort Space Used'], '"NN"', false);
      end loop;
    end loop;
    nodes := nodes || node;
  end loop;
  return nodes;
end;
$$;
