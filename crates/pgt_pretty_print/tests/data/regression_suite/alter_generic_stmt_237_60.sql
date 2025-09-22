ALTER OPERATOR FAMILY alt_opf18 USING btree
  ADD FUNCTION 6 (int4, int2) btint4skipsupport(internal);
