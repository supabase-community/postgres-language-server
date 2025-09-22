SELECT proname, proisstrict FROM pg_proc
       WHERE oid in ('functest_F_1'::regproc,
                     'functest_F_2'::regproc,
                     'functest_F_3'::regproc,
                     'functest_F_4'::regproc) ORDER BY proname;
