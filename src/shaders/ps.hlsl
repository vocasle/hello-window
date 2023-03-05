#include "base_pass.hlsli"

ps_out main(ps_in pin) {
    ps_out pout = (ps_out)0;
    pout.color = float4(1.0, 1.0, 0, 1.0);
    return pout;
}