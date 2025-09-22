create temp table temp_parted_oncommit_test (a int)
  partition by list (a) on commit drop;
