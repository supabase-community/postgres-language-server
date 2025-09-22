select jsonb_pretty(explain_analyze_inc_sort_nodes_without_memory('select * from (select * from t order by a) s order by a, b limit 70'));
