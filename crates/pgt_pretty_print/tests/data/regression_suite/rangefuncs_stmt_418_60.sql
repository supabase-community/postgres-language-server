select *, row_to_json(u) from unnest(array[null::rngfunc2, null::rngfunc2]) u;
