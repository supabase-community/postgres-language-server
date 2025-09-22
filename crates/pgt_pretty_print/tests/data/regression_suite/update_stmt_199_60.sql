insert into utrtest values (2, 'bar')
  returning *, tableoid::regclass;
