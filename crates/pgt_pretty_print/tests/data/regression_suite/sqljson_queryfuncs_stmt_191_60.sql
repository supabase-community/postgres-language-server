SELECT JSON_QUERY(jsonb '[1,2,null,"a"]', '$[*]' RETURNING int[] WITH WRAPPER);
