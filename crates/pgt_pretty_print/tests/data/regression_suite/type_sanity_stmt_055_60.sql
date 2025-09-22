SELECT r.rngtypid, r.rngsubtype
FROM pg_range as r
WHERE r.rngtypid = 0 OR r.rngsubtype = 0 OR r.rngsubopc = 0;
