create trigger base_tab_def_view_instrig instead of insert on base_tab_def_view
  for each row execute function base_tab_def_view_instrig_func();
