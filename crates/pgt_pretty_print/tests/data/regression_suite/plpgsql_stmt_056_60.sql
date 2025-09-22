create trigger tg_chkslotlink before insert or update
    on IFace for each row execute procedure tg_chkslotlink();
