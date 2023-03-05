#include "base_pass.hlsli"

ps_in main(vs_in vin) {
    ps_in vout = (ps_in)0;
    vout.position = float4(vin.position, 1.0);
    return vout;
}