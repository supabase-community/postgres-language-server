select description, inbytes, (test_conv(inbytes, 'mule_internal', 'mule_internal')).* from mic_inputs;
