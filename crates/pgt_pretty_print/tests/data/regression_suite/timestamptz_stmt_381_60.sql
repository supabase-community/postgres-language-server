CREATE VIEW timestamp_local_view AS
  SELECT CAST('1978-07-07 19:38 America/New_York' AS TIMESTAMP WITH TIME ZONE) AT LOCAL AS ttz_at_local,
         timezone(CAST('1978-07-07 19:38 America/New_York' AS TIMESTAMP WITH TIME ZONE)) AS ttz_func,
         TIMESTAMP '1978-07-07 19:38' AT LOCAL AS t_at_local,
         timezone(TIMESTAMP '1978-07-07 19:38') AS t_func;
