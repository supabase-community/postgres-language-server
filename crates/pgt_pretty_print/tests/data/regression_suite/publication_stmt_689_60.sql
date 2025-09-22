CREATE PUBLICATION dump_pub_qual_2ct FOR
  TABLE ONLY pubme.t0 (c) WHERE (c > 0),
  TABLE ONLY pubme.t1 (c);
