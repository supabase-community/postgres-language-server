SELECT * FROM nocols n, LATERAL (VALUES(n.*)) v;
