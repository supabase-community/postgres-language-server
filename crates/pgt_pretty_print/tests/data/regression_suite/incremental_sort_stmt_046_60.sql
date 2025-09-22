select explain_analyze_inc_sort_nodes_verify_invariants('select * from (select * from t order by a) s order by a, b limit 70');
