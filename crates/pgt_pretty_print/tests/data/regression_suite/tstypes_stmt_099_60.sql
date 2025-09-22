SELECT 'wa:1A wb:2D'::tsvector @@ 'w:*D <-> w:*A'::tsquery as "false";
