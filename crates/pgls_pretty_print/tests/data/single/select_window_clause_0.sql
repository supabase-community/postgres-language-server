SELECT total, running_total FROM metrics WHERE total > 0 WINDOW w AS (PARTITION BY series_id ORDER BY captured_at);
