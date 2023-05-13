use async_std::task::block_on;
use interactsh_rs::client::{ClientBuilder, RegisteredClient};
use interactsh_rs::interaction_log::{LogEntry, ParsedLogEntry};
use mlua::Value::Nil;
use mlua::{prelude::LuaResult, Lua, Table, UserData};
use std::time::Duration;
use tealr::TypeName;

#[derive(TypeName, Clone)]
struct OASTWrapper(RegisteredClient);

fn convert_entry_table<'lua>(lua: &'lua Lua, log_entry: &ParsedLogEntry) -> LuaResult<Table<'lua>> {
    let results_table = lua.create_table().unwrap();
    match log_entry {
        ParsedLogEntry::Http {
            unique_id,
            full_id,
            raw_request,
            raw_response,
            remote_address,
            timestamp,
        } => {
            results_table
                .set("unique_id", unique_id.to_string())
                .unwrap();
            results_table.set("full_id", full_id.to_string()).unwrap();
            results_table
                .set("raw_request", raw_request.to_string())
                .unwrap();
            results_table
                .set("raw_response", raw_response.to_string())
                .unwrap();
            results_table
                .set("remote_address", remote_address.to_string())
                .unwrap();
            results_table
                .set("timestamp", timestamp.to_string())
                .unwrap();
        }
        ParsedLogEntry::Dns {
            unique_id,
            full_id,
            q_type,
            raw_request,
            raw_response,
            remote_address,
            timestamp,
        } => {
            results_table
                .set("q_type", q_type.as_ref().unwrap().to_string())
                .unwrap();
            results_table
                .set("unique_id", unique_id.to_string())
                .unwrap();
            results_table.set("full_id", full_id.to_string()).unwrap();
            results_table
                .set("raw_request", raw_request.to_string())
                .unwrap();
            results_table
                .set("raw_response", raw_response.to_string())
                .unwrap();
            results_table
                .set("remote_address", remote_address.to_string())
                .unwrap();
            results_table
                .set("timestamp", timestamp.to_string())
                .unwrap();
        }
        ParsedLogEntry::Ldap {
            unique_id,
            full_id,
            raw_request,
            raw_response,
            remote_address,
            timestamp,
        } => {
            results_table
                .set("unique_id", unique_id.to_string())
                .unwrap();
            results_table.set("full_id", full_id.to_string()).unwrap();
            results_table
                .set("raw_request", raw_request.to_string())
                .unwrap();
            results_table
                .set("raw_response", raw_response.to_string())
                .unwrap();
            results_table
                .set("remote_address", remote_address.to_string())
                .unwrap();
            results_table
                .set("timestamp", timestamp.to_string())
                .unwrap();
        }
        ParsedLogEntry::Smb {
            raw_request,
            timestamp,
        } => {
            results_table
                .set("raw_request", raw_request.to_string())
                .unwrap();
            results_table
                .set("timestamp", timestamp.to_string())
                .unwrap();
        }
        ParsedLogEntry::Ftp {
            remote_address,
            raw_request,
            timestamp,
        } => {
            results_table
                .set("raw_request", raw_request.to_string())
                .unwrap();
            results_table
                .set("remote_address", remote_address.to_string())
                .unwrap();
            results_table
                .set("timestamp", timestamp.to_string())
                .unwrap();
        }
        ParsedLogEntry::Smtp {
            unique_id,
            full_id,
            raw_request,
            smtp_from,
            remote_address,
            timestamp,
        } => {
            results_table
                .set("unique_id", unique_id.to_string())
                .unwrap();
            results_table.set("full_id", full_id.to_string()).unwrap();
            results_table
                .set("smtp_from", smtp_from.to_string())
                .unwrap();
            results_table
                .set("raw_request", raw_request.to_string())
                .unwrap();
            results_table
                .set("remote_address", remote_address.to_string())
                .unwrap();
            results_table
                .set("timestamp", timestamp.to_string())
                .unwrap();
        }
    };
    Ok(results_table)
}

impl UserData for OASTWrapper {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("host", |_, this, ()| Ok(this.0.get_interaction_fqdn()));

        methods.add_method("poll", |lua, this, ()| {
            let poll_results = block_on(this.0.poll()).unwrap();
            let results_table = lua.create_table().unwrap();
            if let Some(poll_results) = poll_results {
                poll_results.iter().for_each(|log_entry| {
                    if let LogEntry::ParsedLog(log_entry) = log_entry {
                        let testing_also = convert_entry_table(lua, log_entry).unwrap();
                        testing_also
                            .clone()
                            .pairs::<String, String>()
                            .for_each(|table_data| {
                                let data = table_data.unwrap();
                                results_table.set(data.0, data.1).unwrap();
                            });
                    }
                });
            }
            Ok(results_table)
        });
    }
}

#[derive(TypeName)]
struct ClientOpts {
    server: Option<String>,
    timeout: Option<u64>,
}

impl Default for ClientOpts {
    fn default() -> Self {
        Self {
            server: None,
            timeout: Some(30),
        }
    }
}

async fn build_client(server_opts: ClientOpts) -> LuaResult<OASTWrapper> {
    let timeout = server_opts.timeout.unwrap_or_else(|| 30);

    let mut client = ClientBuilder::default().parse_logs(true);
    if let Some(server) = server_opts.server {
        client = client.with_server(server);
    }
    let unregistered = client
        .with_timeout(Duration::from_secs(timeout))
        .build()
        .unwrap();

    let registered = unregistered.register().await.unwrap();
    Ok(OASTWrapper(registered))
}

#[mlua::lua_module]
fn interactsh(lua_env: &Lua) -> LuaResult<Table> {
    let the_mod = lua_env.create_table().unwrap();
    the_mod.set(
        "client",
        lua_env.create_async_function(|lua, server_opts: Option<Table>| async move {
            // Convert the table to an Option<Table>
            let server_opts = if let Some(server_opts) = server_opts {
                server_opts
            } else {
                let default_table = lua.create_table().unwrap();
                default_table.set("server", Nil)?;
                default_table.set("timeout", 30)?;
                default_table
            };

            // Extract values from the server_opts table
            let server = server_opts.get::<_, Option<String>>("server")?;
            let timeout = server_opts.get::<_, Option<u64>>("timeout")?;

            // Create a new ClientOpts with the extracted values and defaults
            let mut client_opts = ClientOpts::default();
            client_opts.server = server.or_else(|| client_opts.server.clone());
            client_opts.timeout = timeout.or_else(|| client_opts.timeout.clone());

            let client = build_client(client_opts).await?;
            Ok(client)
        })?,
    )?;
    Ok(the_mod)
}
