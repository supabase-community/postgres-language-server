select
  version(),
  current_setting('server_version_num') :: int8 AS version_num,
  current_setting('server_version_num') :: int8 / 10000 AS major_version,
  (
    select
      count(*) :: int8 AS active_connections
    FROM
      pg_stat_activity
  ) AS active_connections,
  current_setting('max_connections') :: int8 AS max_connections;
