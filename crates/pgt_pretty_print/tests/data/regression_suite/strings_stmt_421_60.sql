SELECT encode(decode(encode(E'\\x00010203', 'base64url'), 'base64url'), 'base64url');
