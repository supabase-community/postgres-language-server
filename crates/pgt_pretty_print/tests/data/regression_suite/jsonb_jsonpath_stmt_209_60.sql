select jsonb '[1,2,3,{"b": [3,4,5]}]' @? 'lax $.*';
