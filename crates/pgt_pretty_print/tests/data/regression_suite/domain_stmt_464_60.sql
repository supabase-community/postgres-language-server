SELECT * FROM information_schema.column_domain_usage
  WHERE domain_name IN ('con', 'dom', 'pos_int', 'things')
  ORDER BY domain_name;
