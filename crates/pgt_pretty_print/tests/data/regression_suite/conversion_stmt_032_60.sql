select description, inbytes, (test_conv(inbytes, 'utf8', 'gb18030')).* from utf8_inputs;
