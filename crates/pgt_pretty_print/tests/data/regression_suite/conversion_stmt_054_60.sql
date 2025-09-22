select description, inbytes, (test_conv(inbytes, 'big5', 'big5')).* from big5_inputs;
