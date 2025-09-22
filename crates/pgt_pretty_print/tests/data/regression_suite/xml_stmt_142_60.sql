CREATE VIEW xmlview10 AS SELECT xmlserialize(document '<foo><bar>42</bar></foo>' AS text indent);
