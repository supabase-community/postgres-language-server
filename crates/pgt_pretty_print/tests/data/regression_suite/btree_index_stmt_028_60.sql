SELECT proname, proargtypes, pronamespace
   FROM pg_proc
   WHERE (proname, pronamespace) > ('abs', 0)
ORDER BY proname, proargtypes, pronamespace LIMIT 1;
