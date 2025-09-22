select description, inbytes, (test_conv(inbytes, 'gb18030', 'utf8')).* from gb18030_inputs;
