CREATE FUNCTION volfoo(text) returns foodomain as
  'begin return $1::foodomain; end' language plpgsql volatile;
