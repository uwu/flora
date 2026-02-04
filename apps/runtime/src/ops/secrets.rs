use deno_core::{OpState, op2};
use deno_error::JsErrorBox;

use crate::secrets::SecretsRuntimeData;

#[op2]
#[string]
pub fn op_secret_placeholder(
    state: &mut OpState,
    #[string] name: String,
) -> Result<Option<String>, JsErrorBox> {
    let secrets = state
        .try_borrow::<std::sync::Arc<SecretsRuntimeData>>()
        .ok_or_else(|| JsErrorBox::generic("secrets unavailable"))?;
    Ok(secrets.placeholder_for(&name))
}
