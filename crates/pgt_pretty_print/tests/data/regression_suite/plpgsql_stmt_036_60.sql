create trigger tg_iface_biu before insert or update
    on IFace for each row execute procedure tg_iface_biu();
