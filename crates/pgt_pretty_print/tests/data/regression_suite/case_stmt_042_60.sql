CREATE FUNCTION vol(text) returns text as
  'begin return $1; end' language plpgsql volatile;
