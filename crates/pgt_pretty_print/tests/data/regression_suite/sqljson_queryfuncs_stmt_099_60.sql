select json_value('{"a": 1.234}', '$.a' returning int error on error);
