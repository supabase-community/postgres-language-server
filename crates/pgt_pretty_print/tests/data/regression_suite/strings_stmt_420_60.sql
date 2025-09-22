SELECT encode(decode(encode(E'\\x000102', 'base64url'), 'base64url'), 'base64url');
