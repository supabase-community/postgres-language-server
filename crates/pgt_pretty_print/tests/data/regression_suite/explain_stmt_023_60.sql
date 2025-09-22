select explain_filter('explain (analyze, generic_plan) select unique1 from tenk1 where thousand = $1');
