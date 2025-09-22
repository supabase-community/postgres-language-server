CREATE VIEW vac_option_tab_counts AS
  SELECT CASE WHEN c.relname IS NULL
    THEN 'main' ELSE 'toast' END as rel,
  s.vacuum_count
  FROM pg_stat_all_tables s
  LEFT JOIN pg_class c ON s.relid = c.reltoastrelid
  WHERE c.relname = 'vac_option_tab' OR s.relname = 'vac_option_tab'
  ORDER BY rel;
