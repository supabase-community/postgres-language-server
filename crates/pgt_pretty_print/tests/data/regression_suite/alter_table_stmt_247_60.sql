alter table nv_parent add check (d between '2001-01-01'::date and '2099-12-31'::date) not valid;
