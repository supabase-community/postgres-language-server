SELECT hashfloat4('NaN'::float4) = hashfloat4(-'NaN'::float4) AS t;
