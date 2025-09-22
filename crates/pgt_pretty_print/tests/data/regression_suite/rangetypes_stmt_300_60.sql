create index on test_range_elem using spgist(int4range(i,i+10));
