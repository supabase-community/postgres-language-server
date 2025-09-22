SELECT stxname, stxdndistinct, stxddependencies, stxdmcv, stxdinherit
  FROM pg_statistic_ext s LEFT JOIN pg_statistic_ext_data d ON (d.stxoid = s.oid)
 WHERE s.stxname = 'ab1_a_b_stats';
