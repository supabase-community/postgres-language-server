select
  query,
  js->0->'Plan'->'Plans'->0->'Actual Rows' as "return by index",
  js->0->'Plan'->'Rows Removed by Index Recheck' as "removed by recheck",
  (res_index = res_heap) as "match"
from
  (values
    ($$ i @> '{}' $$),
    ($$ j @> '{}' $$),
    ($$ i @> '{}' and j @> '{}' $$),
    ($$ i @> '{1}' $$),
    ($$ i @> '{1}' and j @> '{}' $$),
    ($$ i @> '{1}' and i @> '{}' and j @> '{}' $$),
    ($$ j @> '{10}' $$),
    ($$ j @> '{10}' and i @> '{}' $$),
    ($$ j @> '{10}' and j @> '{}' and i @> '{}' $$),
    ($$ i @> '{1}' and j @> '{10}' $$)
  ) q(query),
  lateral explain_query_json($$select * from t_gin_test_tbl where $$ || query) js,
  lateral execute_text_query_index($$select string_agg((i, j)::text, ' ') from t_gin_test_tbl where $$ || query) res_index,
  lateral execute_text_query_heap($$select string_agg((i, j)::text, ' ') from t_gin_test_tbl where $$ || query) res_heap;
