CREATE TRIGGER joinview_upd_trig INSTEAD OF UPDATE ON joinview
  FOR EACH ROW EXECUTE FUNCTION joinview_upd_trig_fn();
