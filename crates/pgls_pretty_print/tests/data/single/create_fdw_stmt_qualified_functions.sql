-- A foreign-data wrapper's handler or validator is a qualified function name.
CREATE FOREIGN DATA WRAPPER f HANDLER te.t_fdw_handler VALIDATOR s.v;
