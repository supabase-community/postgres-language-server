CREATE FUNCTION make_ad(int,int) returns arrdomain as
  'declare x arrdomain;
   begin
     x := array[$1,$2];
     return x;
   end' language plpgsql volatile;
