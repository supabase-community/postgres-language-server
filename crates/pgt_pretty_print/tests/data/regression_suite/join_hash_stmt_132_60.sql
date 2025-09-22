select * from hash_join_batches(
$$
  select count(*) from simple r join simple s using (id);
$$);
