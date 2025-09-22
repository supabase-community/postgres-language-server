SELECT * FROM information_schema.check_constraints
  WHERE (constraint_schema, constraint_name)
        IN (SELECT constraint_schema, constraint_name
            FROM information_schema.domain_constraints
            WHERE domain_name IN ('con', 'dom', 'pos_int', 'things'))
  ORDER BY constraint_name;
