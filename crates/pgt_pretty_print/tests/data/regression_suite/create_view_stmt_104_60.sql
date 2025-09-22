SELECT relname, relkind, reloptions FROM pg_class
       WHERE oid in ('mysecview1'::regclass, 'mysecview2'::regclass,
                     'mysecview3'::regclass, 'mysecview4'::regclass,
                     'mysecview7'::regclass, 'mysecview8'::regclass,
                     'mysecview9'::regclass)
       ORDER BY relname;
