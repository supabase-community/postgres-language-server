SELECT proname, proargtypes, pronamespace
   FROM pg_proc
   WHERE proname = 'zzzzzz' AND (proname, proargtypes) > ('abs', NULL)
   AND pronamespace IN (1, 2, 3) AND proargtypes IN ('26 23', '5077')
ORDER BY proname, proargtypes, pronamespace;
