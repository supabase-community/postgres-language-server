SELECT m.*
FROM (VALUES (1, 0, 0), (3, 0, 20), (4, 100, 10)) AS v(sid, balance, delta),
LATERAL (SELECT r_action, r_tid, r_balance FROM merge_into_sq_target(sid, balance, delta)) m;
