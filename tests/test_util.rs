use base64::prelude::*;
use httpmock::prelude::HttpMockRequest;
use httpmock::{MockServer, Then, When};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

pub fn setup_config(config_dir: &Path, server_url: &str) -> Option<std::path::PathBuf> {
    let config_path = config_dir.join("config.toml");
    let backup_path = config_dir.join("config_backup.toml");

    if config_path.exists() {
        fs::rename(&config_path, &backup_path).expect("Failed to backup original config.toml");
    }

    fs::create_dir_all(config_dir).expect("Failed to create config directory");
    fs::write(
        &config_path,
        format!(
            r#"
            version = "2"
            credentials_home_dir = "~/.near-credentials"

            [network_connection.mainnet]
            network_name = "mainnet"
            rpc_url = "{}"
            wallet_url = "https://app.mynearwallet.com/"
            explorer_transaction_url = "https://explorer.near.org/transactions/"
            linkdrop_account_id = "near"
            near_social_db_contract_account_id = "social.near"
            fastnear_url = "https://api.fastnear.com/"
            staking_pools_factory_account_id = "poolv1.near"
            coingecko_url = "https://api.coingecko.com/"
            "#,
            server_url
        ),
    )
    .expect("Failed to create test config.toml");

    if backup_path.exists() {
        Some(backup_path)
    } else {
        None
    }
}

pub fn restore_config(config_dir: &Path, backup_path: Option<std::path::PathBuf>) {
    let config_path = config_dir.join("config.toml");
    if let Some(backup_path) = backup_path {
        fs::rename(backup_path, config_path).expect("Failed to restore original config.toml");
    } else {
        fs::remove_file(&config_path).expect("Failed to remove generated config.toml");
    }
}

pub fn setup_temp_dir() -> TempDir {
    let temp_dir = tempfile::tempdir().unwrap();
    let src_dir = temp_dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    temp_dir
}

pub fn setup_mock_server() -> MockServer {
    let server = MockServer::start();

    // Mock for view_access_key RPC call
    server.mock(|when: When, then: Then| {
        when.method(httpmock::Method::POST)
            .path("/")
            .body_contains("view_access_key");
        then.status(200).json_body(json!({
            "jsonrpc": "2.0",
            "result": {
                "nonce": 85,
                "permission": {
                    "FunctionCall": {
                        "allowance": "18501534631167209000000000",
                        "receiver_id": "social.near",
                        "method_names": ["set"]
                    }
                },
                "block_height": 19884918,
                "block_hash": "GGJQ8yjmo7aEoj8ZpAhGehnq9BSWFx4xswHYzDwwAP2n"
            },
            "id": "dontcare"
        }));
    });

    // Mock the RPC call for `view_access_key_list`
    server.mock(|when: When, then: Then| {
        when.method(httpmock::Method::POST)
            .path("/")
            .body_contains(r#""request_type":"view_access_key_list""#);
        then.status(200).json_body(json!({
            "jsonrpc": "2.0",
            "result": {
                "keys": [
                    {
                        "public_key": "ed25519:7fvCiaE4NTmhexo8fDoa3CFNupL6mvJmNjL1hydN65fm",
                        "access_key": {
                            "nonce": 17,
                            "permission": {
                                "FunctionCall": {
                                    "allowance": "9999203942481156415000",
                                    "receiver_id": "social.near",
                                    "method_names": ["set"]
                                }
                            }
                        }
                    },
                    {
                        "public_key": "ed25519:4F9TwuSqWwvoyu7JVZDsupPhC7oYbYNsisBV2yQvyXFn",
                        "access_key": {
                            "nonce": 0,
                            "permission": "FullAccess"
                        }
                    }
                ],
                "block_height": 17798231,
                "block_hash": "Gm7YSdx22wPuciW1jTTeRGP9mFqmon69ErFQvgcFyEEB"
            },
            "id": "dontcare"
        }));
    });

    // Mock the `query` RPC call for getting existing deployed components (returning an empty JSON object as a result)
    server.mock(|when: When, then: Then| {
        when.method(httpmock::Method::POST)
            .path("/")
            .body_contains(r#""method":"query""#)
            .body_contains(r#""request_type":"call_function""#)
            .body_contains(r#""method_name":"get""#);
        then.status(200).json_body(json!({
            "jsonrpc": "2.0",
            "result": {
                "result": [123, 125], // ASCII for `{}` is 123, 125
                "logs": [],
                "block_height": 17817336,
                "block_hash": "4qkA4sUUG8opjH5Q9bL5mWJTnfR4ech879Db1BZXbx6P"
            },
            "id": "dontcare"
        }));
    });

    // Mock the `query` RPC call for `storage_balance_of`
    server.mock(|when: When, then: Then| {
        when.method(httpmock::Method::POST)
            .path("/")
            .body_contains(r#""method":"query""#)
            .body_contains(r#""request_type":"call_function""#)
            .body_contains(r#""method_name":"storage_balance_of""#);

        // Create the JSON object
        let balance_json = json!({
            "available": "17413620000000000000000015",
            "total": "27100000000000000000000015"
        });

        // Serialize to a string and convert to ASCII character codes
        let balance_string = balance_json.to_string();
        let result: Vec<u8> = balance_string.bytes().collect();

        then.status(200).json_body(json!({
            "jsonrpc": "2.0",
            "result": {
                "result": result,
                "logs": [],
                "block_height": 17817337,
                "block_hash": "6qkA4sUUG8opjH5Q9bL5mWJTnfR4ech879Db1BZXbx7Q"
            },
            "id": "dontcare"
        }));
    });

    let write_permission = false;
    server.mock(|when, then| {
        when.body_contains("is_write_permission_granted");
        let write_permission_json_str = serde_json::to_string(&json!(write_permission)).unwrap();
        let binary_write_permission = write_permission_json_str.as_bytes().to_vec();
        then.json_body(json!({
          "jsonrpc": "2.0",
          "result": {
            "result": binary_write_permission,
            "logs": [],
            "block_height": 17817336,
            "block_hash": "4qkA4sUUG8opjH5Q9bL5mWJTnfR4ech879Db1BZXbx6P"
          },
          "id": "dontcare"
        }));
    });

    server
}

pub fn mock_unmatched(server: MockServer) -> MockServer {
    // Log unmatched requests and return a 500 error
    server.mock(|when: When, then: Then| {
        when.matches(|req| {
            if let Some(body_bytes) = &req.body {
                // Convert body to string
                let body_str = String::from_utf8_lossy(body_bytes);
                if let Ok(json_body) = serde_json::from_str::<Value>(&body_str) {
                    println!(
                        "No mock for request: {}",
                        serde_json::to_string_pretty(&json_body).unwrap()
                    );
                } else {
                    println!("Failed to parse JSON body");
                }
            }
            true
        });
        then.status(500);
    });
    server
}

pub fn mock_broadcast_tx_commit(server: MockServer, component_content: &str, matcher: fn(&HttpMockRequest) -> bool) -> MockServer {
    let expected_args_base64 = BASE64_STANDARD.encode(format!(
        r#"{{"data":{{"test.near":{{"widget":{{"example_component":{{"":{}}}}}}}}}}}"#,
        serde_json::to_string(component_content).unwrap()
    ));

    server.mock(move |when: When, then: Then| {
        
        when.method(httpmock::Method::POST)
            .path("/")
            .matches(matcher);

        then.status(200).json_body(json!({
            "jsonrpc": "2.0",
            "result": {
                "final_execution_status": "FINAL",
                "status": {
                    "SuccessValue": ""
                },
                "transaction": {
                    "signer_id": "test.near",
                    "receiver_id": "social.near",
                    "public_key": "ed25519:7fvCiaE4NTmhexo8fDoa3CFNupL6mvJmNjL1hydN65fm",
                    "priority_fee": 0,
                    "signature": "ed25519:7oCBMfSHrZkT7tzPDBxxCd3tWFhTES38eks3MCZMpYPJRfPWKxJsvmwQiVBBxRLoxPTnXVaMU2jPV3MdFKZTobH",
                    "nonce": 13,
                    "actions": [{
                        "FunctionCall": {
                            "method_name": "set",
                            "gas": 0,
                            "deposit": "0",
                            "args": expected_args_base64
                        }
                    }],
                    "hash": "ASS7oYwGiem9HaNwJe6vS2kznx2CxueKDvU9BAYJRjNR"
                },
                "transaction_outcome": {
                "proof": [],
                "block_hash": "9MzuZrRPW1BGpFnZJUJg6SzCrixPpJDfjsNeUobRXsLe",
                "id": "ASS7oYwGiem9HaNwJe6vS2kznx2CxueKDvU9BAYJRjNR",
                "outcome": {
                    "logs": [],
                    "receipt_ids": ["BLV2q6p8DX7pVgXRtGtBkyUNrnqkNyU7iSksXG7BjVZh"],
                    "gas_burnt": 1,
                    "tokens_burnt": "22318256250000000000",
                    "executor_id": "sender.testnet",
                    "status": {
                    "SuccessReceiptId": "BLV2q6p8DX7pVgXRtGtBkyUNrnqkNyU7iSksXG7BjVZh"
                    }
                }
                },
                "receipts_outcome": [
                    {
                        "proof": [],
                        "block_hash": "5Hpj1PeCi32ZkNXgiD1DrW4wvW4Xtic74DJKfyJ9XL3a",
                        "id": "BLV2q6p8DX7pVgXRtGtBkyUNrnqkNyU7iSksXG7BjVZh",
                        "outcome": {
                            "logs": [],
                            "receipt_ids": ["3sawynPNP8UkeCviGqJGwiwEacfPyxDKRxsEWPpaUqtR"],
                            "gas_burnt": 1,
                            "tokens_burnt": "22318256250000000000",
                            "executor_id": "receiver.testnet",
                            "status": {
                            "SuccessValue": ""
                            }
                        }
                        }
                ]
            },
            "id": "dontcare"
        }));
    });
    server
}

pub fn match_broadcast_tx_commit_for_component_content(body: &[u8], component_content: &str) -> bool {
    let body_str = String::from_utf8_lossy(body);

    if let Ok(json_body) = serde_json::from_str::<Value>(&body_str) {
        if let Some(params) = json_body.get("params").and_then(|p| p.get(0)) {
            if let Some(params_str) = params.as_str() {
                let decoded_params = BASE64_STANDARD
                    .decode(params_str)
                    .expect("Failed to decode base64");

                let decoded_str = String::from_utf8_lossy(&decoded_params);

                let expected_data = format!(
                    r#"{{"data":{{"test.near":{{"widget":{{"example_component":{{"":{}}}}}}}}}"#,
                    serde_json::to_string(&component_content).unwrap()
                );

                return decoded_str.contains(&expected_data);
            }
        }
    }
    false
}
