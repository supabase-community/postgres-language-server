SELECT encode(('\x' || repeat('1234567890abcdef0001', 7))::bytea, 'base64');
