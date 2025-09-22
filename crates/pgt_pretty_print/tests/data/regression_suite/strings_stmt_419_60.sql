SELECT encode(decode(encode(E'\\x0001', 'base64url'), 'base64url'), 'base64url');
