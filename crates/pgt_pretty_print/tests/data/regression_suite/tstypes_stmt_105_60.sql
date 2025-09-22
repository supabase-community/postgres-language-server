SELECT strip('wa:1A'::tsvector) @@ 'w:*D'::tsquery as "true";
