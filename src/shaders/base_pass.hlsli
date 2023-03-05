struct vs_in {
    float3 position : POSITION;
};

struct ps_in {
    float4 position : SV_POSITION;
};

struct ps_out {
    float4 color : SV_TARGET;
};