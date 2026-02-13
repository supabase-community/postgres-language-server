SELECT *
FROM JSON_TABLE(
    '{"employees":[{"name":"Al","age":1}]}'::jsonb,
    '$.employees[*]'
    COLUMNS (
        name text PATH '$.name',
        age int PATH '$.age'
    )
) AS jt;
