CREATE ROLE regress_maintain;

SET ROLE regress_maintain;

CREATE TEMP TABLE past_inh_db_other ();

CREATE TEMP TABLE past_inh_db_child () INHERITS (past_inh_db_parent);

CREATE INDEX ON past_inh_db_parent ((1));

ANALYZE past_inh_db_parent;

SELECT reltuples, relhassubclass
  FROM pg_class WHERE oid = 'past_inh_db_parent'::regclass;

DROP TABLE past_inh_db_child;

SET client_min_messages = error;

ANALYZE;

RESET client_min_messages;

SELECT reltuples, relhassubclass
  FROM pg_class WHERE oid = 'past_inh_db_parent'::regclass;

DROP TABLE past_inh_db_parent, past_inh_db_other;

RESET ROLE;

DROP ROLE regress_maintain;
