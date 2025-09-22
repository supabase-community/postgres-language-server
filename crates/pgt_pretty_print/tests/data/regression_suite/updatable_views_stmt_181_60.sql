UPDATE rw_view2 SET b='Row three' WHERE a=3 RETURNING old.*, new.*;
