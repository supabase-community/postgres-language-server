prepare mt_q1 (int) as select a from ma_test where a >= $1 and a % 10 = 5 order by b;
