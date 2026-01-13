use deno_core::{OpState, ResourceId, op2};

#[op2]
#[serde]
pub fn op_tls_peer_certificate(
    _state: &mut OpState,
    #[smi] _rid: ResourceId,
    _detailed: bool,
) -> Option<serde_json::Value> {
    None
}
