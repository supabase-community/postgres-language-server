select description, inbytes, (test_conv(inbytes, 'utf8', 'latin5')).* from utf8_inputs;
