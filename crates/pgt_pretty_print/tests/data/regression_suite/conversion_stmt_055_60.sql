select description, inbytes, (test_conv(inbytes, 'big5', 'utf8')).* from big5_inputs;
