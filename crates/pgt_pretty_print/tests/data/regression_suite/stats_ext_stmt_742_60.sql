SELECT * FROM tststats.priv_test_view t
 WHERE a <<< 0 AND (b <<< 0 OR t.* <<< (1, 1) IS NOT NULL);
