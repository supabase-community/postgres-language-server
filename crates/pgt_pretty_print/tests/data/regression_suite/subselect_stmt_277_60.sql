select * from json_tab t1 left join (select json_array(1, a) from json_tab t2) s on false;
