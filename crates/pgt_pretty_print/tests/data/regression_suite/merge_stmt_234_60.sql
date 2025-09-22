UPDATE sq_target SET balance = balance + 1 RETURNING merge_action();
