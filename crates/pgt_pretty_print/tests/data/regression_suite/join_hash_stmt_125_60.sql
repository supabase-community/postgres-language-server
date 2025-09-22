select * from hash_join_batches(
$$
  select count(*) from simple r join extremely_skewed s using (id);
$$);
