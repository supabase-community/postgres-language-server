create function mr_polymorphic(i anyrange) returns anymultirange
  as $$ begin return multirange($1); end; $$ language plpgsql;
