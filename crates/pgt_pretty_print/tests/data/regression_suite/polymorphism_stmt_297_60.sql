create function xleast(x numeric, variadic arr numeric[])
  returns numeric as $$
  select least(x, min(arr[i])) from generate_subscripts(arr, 1) g(i);
$$ language sql;
