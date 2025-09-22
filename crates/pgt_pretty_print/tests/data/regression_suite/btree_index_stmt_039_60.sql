SELECT proname, proargtypes, pronamespace
   FROM pg_proc
   WHERE (proname, proargtypes) > ('abs', NULL) AND proname <= 'abs'
ORDER BY proname DESC, proargtypes DESC, pronamespace DESC;
