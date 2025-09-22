CREATE PUBLICATION dump_pub_all FOR
  TABLE ONLY pubme.t0,
  TABLE ONLY pubme.t1 WHERE (c < 0),
  TABLES IN SCHEMA pubme,
  TABLES IN SCHEMA pubme2
  WITH (publish_via_partition_root = true);
