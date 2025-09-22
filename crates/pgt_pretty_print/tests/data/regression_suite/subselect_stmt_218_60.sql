select x, x from
    (select (select now() where y=y) as x from (values(1),(2)) v(y)) ss;
