create trigger tg_chkslotname before insert
    on IFace for each row execute procedure tg_chkslotname('IF');
