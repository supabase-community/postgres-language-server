select jsonb_path_query('[null,1,true,"a",[],{}]', '$.type()');
