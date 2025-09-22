select jsonb_path_query('"10-03-2017"', '$.datetime("dd-mm-yyyy").type()');
