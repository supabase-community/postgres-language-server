SELECT strip('wa:1A'::tsvector) @@ 'w:*A'::tsquery as "true";
