-- implicit arguments in order, an index sets the next one, `*` takes one
select %v, %v, %[1]s, %s, %[1]d, %*d, %d from t;
select %[2]s, %s, %[1]s, %.*f, %[3]*.[2]*[1]f;
