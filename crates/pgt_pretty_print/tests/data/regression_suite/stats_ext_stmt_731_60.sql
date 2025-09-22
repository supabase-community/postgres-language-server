CREATE OPERATOR <<< (procedure = op_leak, leftarg = record, rightarg = record,
                     restrict = scalarltsel);
