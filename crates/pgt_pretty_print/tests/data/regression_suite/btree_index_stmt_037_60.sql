SELECT proname, proargtypes, pronamespace
   FROM pg_proc
   WHERE proname >= 'abs' AND (proname, proargtypes) <= ('abs', NULL)
ORDER BY proname DESC, proargtypes DESC, pronamespace DESC;
