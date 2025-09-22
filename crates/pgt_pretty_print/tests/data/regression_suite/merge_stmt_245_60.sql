CREATE FUNCTION merge_into_sq_target(sid int, balance int, delta int,
                                     OUT action text, OUT tid int, OUT new_balance int)
LANGUAGE sql AS
$$
    MERGE INTO sq_target t
    USING (VALUES ($1, $2, $3)) AS v(sid, balance, delta)
    ON tid = v.sid
    WHEN MATCHED AND tid >= 2 THEN
        UPDATE SET balance = t.balance + v.delta
    WHEN NOT MATCHED THEN
        INSERT (balance, tid) VALUES (v.balance + v.delta, v.sid)
    WHEN MATCHED AND tid < 2 THEN
        DELETE
    RETURNING merge_action(), t.*;
$$;
