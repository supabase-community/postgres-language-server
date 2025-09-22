ALTER OPERATOR FAMILY alt_opf18 USING btree
  ADD FUNCTION 4 (int4, int2) btequalimage(oid);
