create view insertconflictv as
  select * from insertconflict with cascaded check option;
