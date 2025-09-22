WITH m AS (
    MERGE INTO sq_target t
    USING sq_source s
    ON tid = sid
    WHEN MATCHED AND tid >= 2 THEN
        UPDATE SET balance = t.balance + delta
    WHEN NOT MATCHED THEN
        INSERT (balance, tid) VALUES (balance + delta, sid)
    WHEN MATCHED AND tid < 2 THEN
        DELETE
    RETURNING merge_action() AS action, old AS old_data, new AS new_data, t.*,
              CASE merge_action()
                  WHEN 'INSERT' THEN 'Inserted '||t
                  WHEN 'UPDATE' THEN 'Added '||delta||' to balance'
                  WHEN 'DELETE' THEN 'Removed '||t
              END AS description
), m2 AS (
    MERGE INTO sq_target_merge_log l
    USING m
    ON l.tid = m.tid
    WHEN MATCHED THEN
        UPDATE SET last_change = description
    WHEN NOT MATCHED THEN
        INSERT VALUES (m.tid, description)
    RETURNING m.*, merge_action() AS log_action, old AS old_log, new AS new_log, l.*
)
SELECT * FROM m2;
