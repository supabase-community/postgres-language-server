create trigger some_tab_stmt_trig
  before update on some_tab execute function some_tab_stmt_trig_func();
