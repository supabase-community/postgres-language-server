SELECT condeferrable, condeferred, conenforced, convalidated
FROM pg_constraint WHERE conname = 'fk_con';
