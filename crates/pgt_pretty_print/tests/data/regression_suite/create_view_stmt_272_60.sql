create rule updlog as on update to tt15v do also
  insert into tt15v_log values(old, new, row(old,old) < row(new,new));
