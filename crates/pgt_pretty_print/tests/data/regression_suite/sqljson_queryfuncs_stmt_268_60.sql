SELECT JSON_QUERY('"a"', '$.a'  RETURNING someparent DEFAULT (SELECT '(1)')::somechild::someparent ON ERROR);
