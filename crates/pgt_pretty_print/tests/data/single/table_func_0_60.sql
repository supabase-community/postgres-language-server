SELECT *
FROM JSON_TABLE(
    '{"employees":[{"name":"John","age":30},{"name":"Jane","age":25}]}'::jsonb,
    '$.employees[*]'
    COLUMNS (
        name text PATH '$.name',
        age int PATH '$.age'
    )
) AS jt;