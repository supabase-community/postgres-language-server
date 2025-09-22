create trigger depth_b_tr before insert on depth_b
  for each row execute procedure depth_b_tf();
