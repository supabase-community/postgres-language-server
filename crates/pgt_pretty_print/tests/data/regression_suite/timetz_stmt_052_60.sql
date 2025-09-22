CREATE VIEW timetz_local_view AS
  SELECT f1 AS dat,
       timezone(f1) AS dat_func,
       f1 AT LOCAL AS dat_at_local,
       f1 AT TIME ZONE current_setting('TimeZone') AS dat_at_tz,
       f1 AT TIME ZONE INTERVAL '00:00' AS dat_at_int
  FROM TIMETZ_TBL
  ORDER BY f1;
