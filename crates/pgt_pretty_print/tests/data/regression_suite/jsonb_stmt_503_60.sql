select *, c is null as c_is_null
from jsonb_to_record('{"a":1, "b":{"c":16, "d":2}, "x":8, "ca": ["1 2", 3], "ia": [[1,2],[3,4]], "r": {"a": "aaa", "b": 123}}'::jsonb)
    as t(a int, b jsonb, c text, x int, ca char(5)[], ia int[][], r jbpop);
