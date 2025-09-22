SELECT op.oprname AS operator_name, com.oprname AS commutator_name,
  com.oprcode AS commutator_func
  FROM pg_operator op
  INNER JOIN pg_operator com ON (op.oid = com.oprcom AND op.oprcom = com.oid)
  WHERE op.oprname = '==='
  AND op.oprleft = 'boolean'::regtype AND op.oprright = 'real'::regtype;
