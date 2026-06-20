use std::sync::Arc;

use anyhow::Result;
use serde_json::json;

use crate::app::AppContext;

pub async fn perform_hover(context: &Arc<AppContext>, action_id: i64, champion_id: i64) -> Result<()> {
    let client = context.lcu_client.read().await.clone();
    let Some(client) = client else {
        return Err(anyhow::anyhow!("LCU client not connected"));
    };

    client
        .patch_json(
            &format!("/lol-champ-select/v1/session/actions/{}", action_id),
            json!({ "championId": champion_id }),
        )
        .await?;

    Ok(())
}
