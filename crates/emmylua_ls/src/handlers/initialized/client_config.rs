use std::{collections::HashMap, time::Duration};

use serde::{Deserialize, Serialize};

use crate::{context::ServerContextSnapshot, util::time_cancel_token};

use super::ClientId;

#[derive(Debug)]
pub struct ClientConfig {
    pub client_id: ClientId,
    pub exclude: Vec<String>,
    pub extensions: Vec<String>,
    pub encoding: String,
}

pub async fn get_client_config(
    context: &ServerContextSnapshot,
    client_id: ClientId,
) -> ClientConfig {
    let mut config = ClientConfig {
        client_id,
        exclude: Vec::new(),
        extensions: Vec::new(),
        encoding: "utf-8".to_string(),
    };
    match client_id {
        ClientId::VSCode => get_client_config_vscode(context, &mut config).await,
        _ => Some(()),
    };
    

    config
}

async fn get_client_config_vscode(context: &ServerContextSnapshot, config:&mut ClientConfig) -> Option<()>{
    let client = &context.client;
    let params = lsp_types::ConfigurationParams {
        items: vec![lsp_types::ConfigurationItem {
            scope_uri: None,
            section: Some("files".to_string()),
        }],
    };
    let cancel_token = time_cancel_token(Duration::from_secs(5));
    let files_configs = client.get_configuration::<VscodeFilesConfig>(params, cancel_token).await?;
    for files_config in files_configs {
        if let Some(exclude) = files_config.exclude {
            for (pattern, _) in exclude {
                config.exclude.push(pattern);
            }
        }
        if let Some(associations) = files_config.associations {
            for (pattern, extension) in associations {
                if extension == "lua" || extension == "Lua" {
                    config.extensions.push(pattern);
                }
            }
        }
        config.encoding = files_config.encoding.unwrap_or("utf-8".to_string());
    }

    Some(())
}

#[derive(Debug, Deserialize, Serialize)]
struct VscodeFilesConfig {
    exclude: Option<HashMap<String, bool>>,
    associations: Option<HashMap<String, String>>,
    encoding: Option<String>
}