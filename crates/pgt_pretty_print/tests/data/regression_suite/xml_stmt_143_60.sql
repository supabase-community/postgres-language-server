CREATE VIEW xmlview11 AS SELECT xmlserialize(document '<foo><bar>42</bar></foo>' AS character varying no indent);
