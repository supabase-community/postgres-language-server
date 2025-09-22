SELECT f1 AS dat,
       f1 AT TIME ZONE 'UTC+10' AS dat_at_tz,
       f1 AT TIME ZONE INTERVAL '-10:00' AS dat_at_int
  FROM TIMETZ_TBL
  ORDER BY f1;
