alter table parted_constr_ancestor attach partition parted_constr
  for values from ('aaaa') to ('zzzz');
