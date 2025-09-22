select description, inbytes, (test_conv(inbytes::text::bytea, 'gb18030', 'gb18030')).* from gb18030_inputs;
