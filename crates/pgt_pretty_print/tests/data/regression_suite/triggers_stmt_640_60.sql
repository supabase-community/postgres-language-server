select tgrelid::regclass, rtrim(tgname, '0123456789') as tgname,
  tgfoid::regproc, tgenabled
  from pg_trigger where tgrelid in ('parent'::regclass, 'child1'::regclass)
  order by tgrelid::regclass::text, tgfoid;
