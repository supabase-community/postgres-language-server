update inhpar i set (f1, f2) = (select i.f1, i.f2 || '-' from int4_tbl limit 1);
