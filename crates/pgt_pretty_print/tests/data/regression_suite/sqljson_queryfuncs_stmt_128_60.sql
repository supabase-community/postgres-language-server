SELECT JSON_QUERY(jsonb'{"rec": "[1,2]"}', '$.rec' returning int4range keep quotes);
