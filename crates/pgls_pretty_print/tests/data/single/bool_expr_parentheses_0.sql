SELECT
  *
FROM
  demo
WHERE
  (flag_a OR flag_b)
  AND NOT (flag_c OR flag_d)
  AND (flag_e AND flag_f OR flag_g);
