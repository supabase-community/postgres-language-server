CREATE PUBLICATION dump_pub_qual_1ct FOR
  TABLE ONLY pubme.t0 (c, d) WHERE (c > 0);
