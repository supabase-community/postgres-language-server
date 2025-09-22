SELECT encode(decode(encode(E'\\x00', 'base64url'), 'base64url'), 'base64url');
