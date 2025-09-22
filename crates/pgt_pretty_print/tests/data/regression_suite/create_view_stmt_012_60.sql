CREATE VIEW key_dependent_view AS
   SELECT * FROM view_base_table GROUP BY key;
