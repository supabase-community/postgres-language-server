SELECT * FROM information_schema.domain_constraints
  WHERE domain_name IN ('con', 'dom', 'pos_int', 'things')
  ORDER BY constraint_name;
