UPDATE foo SET f2 = foo_f.f2 FROM foo_f() WHERE foo_f.f1 = foo.f1
  RETURNING foo_f;
