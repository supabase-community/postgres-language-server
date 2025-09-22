SELECT op.oprname AS operator_name, neg.oprname AS negator_name,
  neg.oprcode AS negator_func
  FROM pg_operator op
  INNER JOIN pg_operator neg ON (op.oid = neg.oprnegate AND op.oprnegate = neg.oid)
  WHERE op.oprname = '==='
  AND op.oprleft = 'boolean'::regtype AND op.oprright = 'real'::regtype;
