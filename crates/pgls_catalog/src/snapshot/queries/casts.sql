select castsource::int8 as "source!", casttarget::int8 as "target!", castcontext::text as "context!", castmethod::text as "method!" from pg_cast;
