SELECT sum(reuses) AS reuses, sum(reads) AS reads, sum(evictions) AS evictions
  FROM pg_stat_io WHERE context = 'vacuum' ;
