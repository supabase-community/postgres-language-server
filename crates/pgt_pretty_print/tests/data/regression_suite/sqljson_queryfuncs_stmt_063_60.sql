SELECT JSON_VALUE('"purple"'::jsonb, 'lax $[*]' RETURNING rgb ERROR ON ERROR);
