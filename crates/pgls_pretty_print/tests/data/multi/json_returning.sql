select json_value(jsonb '123', '$' returning int);

select json_value(jsonb '{"a": {"b": "a rather long string value"}}', '$.a.b' returning text default 'fallback' on empty error on error);

select json_query(jsonb '[1, 2]', '$' returning text);

select json_query(jsonb '{"a": [1, 2, 3, 4, 5, 6, 7, 8]}', '$.a' returning jsonb with conditional wrapper keep quotes);

select json_arrayagg(i returning jsonb) from generate_series(1, 3) i;

select json_arrayagg(i absent on null returning jsonb) from generate_series(1, 3) i;

select json_objectagg(k : v null on null with unique keys returning jsonb) from (values ('a', 1)) t(k, v);

select json_object('a' : 1 returning jsonb), json_array(1, 2 returning jsonb), json_serialize(jsonb '1' returning text);
