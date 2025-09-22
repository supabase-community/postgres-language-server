SELECT r.rngtypid, r.rngsubtype, r.rngmultitypid
FROM pg_range r
WHERE r.rngmultitypid IS NULL OR r.rngmultitypid = 0;
