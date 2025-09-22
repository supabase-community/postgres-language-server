SELECT p.* FROM POINT_TBL p
   WHERE not box '(0,0,100,100)' @> p.f1;
