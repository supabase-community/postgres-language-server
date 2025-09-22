select explain_analyze_without_memory('select * from (select * from t order by a) s order by a, b limit 55');
