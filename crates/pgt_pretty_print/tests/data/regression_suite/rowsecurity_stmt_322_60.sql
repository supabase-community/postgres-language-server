DELETE FROM only t1 WHERE f_leak(b) RETURNING tableoid::regclass, *, t1;
