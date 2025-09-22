SELECT proname, proargtypes, pronamespace
   FROM pg_proc
   WHERE (proname, pronamespace) < ('abs', 1_000_000)
ORDER BY proname DESC, proargtypes DESC, pronamespace DESC LIMIT 1;
