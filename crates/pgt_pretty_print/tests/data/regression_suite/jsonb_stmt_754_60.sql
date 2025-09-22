select jsonb_strip_nulls('{"a": {"b": null, "c": null}, "d": {} }', true);
