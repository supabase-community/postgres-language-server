CREATE FUNCTION merge_sq_source_into_sq_target()
RETURNS TABLE (action text, tid int, balance int)
LANGUAGE sql AS
$$
    MERGE INTO sq_target t
    USING sq_source s
    ON tid = sid
    WHEN MATCHED AND tid >= 2 THEN
        UPDATE SET balance = t.balance + delta
    WHEN NOT MATCHED THEN
        INSERT (balance, tid) VALUES (balance + delta, sid)
    WHEN MATCHED AND tid < 2 THEN
        DELETE
    RETURNING merge_action(), t.*;
$$;
