SELECT JSON_QUERY(jsonb'{"rec": "{1,2,3}"}', '$.rec' returning int[] keep quotes error on error);
