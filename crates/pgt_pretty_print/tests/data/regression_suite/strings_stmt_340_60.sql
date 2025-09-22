SELECT pg_column_compression(f1) AS f1_comp, pg_column_compression(f2) AS f2_comp
  FROM toasttest;
