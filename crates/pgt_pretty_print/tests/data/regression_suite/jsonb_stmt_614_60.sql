SELECT distinct * FROM (values (jsonb '{}' || ''::text),('{}')) v(j);
