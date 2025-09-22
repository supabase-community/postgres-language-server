prepare mt_q2 (int) as select * from ma_test where a >= $1 order by b limit 1;
