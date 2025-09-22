SELECT encode(decode(encode(E'\\x', 'base64url'), 'base64url'), 'base64url');
