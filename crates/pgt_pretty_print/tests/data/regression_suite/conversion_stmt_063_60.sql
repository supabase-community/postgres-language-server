select description, inbytes, (test_conv(inbytes, 'mule_internal', 'big5')).* from mic_inputs;
