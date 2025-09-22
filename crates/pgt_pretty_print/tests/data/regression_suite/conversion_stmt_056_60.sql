select description, inbytes, (test_conv(inbytes, 'big5', 'mule_internal')).* from big5_inputs;
