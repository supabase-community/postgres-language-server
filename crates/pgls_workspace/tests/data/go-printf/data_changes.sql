insert into   %s (a, b) values (%d, %v);
update %s set   a = %d, b = %v where id in (%d, %d);
delete from   %s where id = %d returning *;
