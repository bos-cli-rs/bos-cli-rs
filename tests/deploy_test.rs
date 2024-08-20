// tests/deploy_test.rs

use assert_cmd::Command;
use httpmock::Method::POST;
use httpmock::MockServer;
use std::fs;
use std::path::Path;
use tempfile::tempdir;
use serde_json::json;

#[test]
fn test_bos_components_deploy_with_mocked_rpc() {
    // Step 1: Start a mock server to simulate the NEAR RPC server
    let server = MockServer::start();

    // Step 2: Set up a temporary directory for the modified config
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().join("near-cli");
    fs::create_dir(&config_dir).unwrap();

    // Step 3: Write the modified config.toml with the mock server's URL
    let config_content = format!(
        r#"
        version = "2"
        credentials_home_dir = "{}/.near-credentials"

        [network_connection.mainnet]
        network_name = "mainnet"
        rpc_url = "{}"
        wallet_url = "https://wallet.near.org/"
        explorer_transaction_url = "https://explorer.near.org/transactions/"
        linkdrop_account_id = "near"
        fastnear_url = "https://api.fastnear.com/"
        staking_pools_factory_account_id = "poolv1.near"
        coingecko_url = "https://api.coingecko.com/"

        "#,
        temp_dir.path().display(),
        server.url("/"),  // Pointing to the mock server
    );

    let config_path = config_dir.join("config.toml");
    fs::write(&config_path, config_content).unwrap();

    // Step 6: Set the NEAR CLI config directory to the temporary directory
    std::env::set_var("NEAR_CLI_HOME", &config_dir);

    // **Debug Step**: Print out the NEAR_CLI_HOME environment variable
    println!("NEAR_CLI_HOME: {}", std::env::var("NEAR_CLI_HOME").unwrap());

    // **Debug Step**: Print the path to the config file being used
    println!("Config file path: {}", config_path.display());

    // **Debug Step**: Print the contents of the config file to ensure it's correct
    let config_contents = fs::read_to_string(&config_path).unwrap();
    println!("Config file contents:\n{}", config_contents);

    // Step 4: Mock the necessary RPC calls on the mock server
    let _mock = server.mock(|when, then| {
        when.method(POST)
            .path("/")
            .body_contains("view_access_key");
        then.status(200)
            .json_body(json!({
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

    // Step 4: Mock the RPC call for `view_access_key_list`
    server.mock(|when, then| {
        when.method(POST)
            .path("/")
            .body_contains(r#""request_type":"view_access_key_list""#);
        then.status(200)
            .json_body(json!({
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

    let src_dir = temp_dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    // Create a mock component file
    let component_path = src_dir.join("example_component.jsx");
    fs::write(&component_path, "console.log('Hello, world!');").unwrap();

    // Step 5: Change the current directory to the temporary directory
    std::env::set_current_dir(&temp_dir).unwrap();

    // Step 7: Run the CLI command as a subprocess
    let mut cmd = Command::cargo_bin("bos").unwrap();

    cmd.args(&[
        "components",
        "deploy",
        "test.near",
        "sign-as",
        "test.near",
        "network-config",
        "mainnet",
        "sign-with-plaintext-private-key",
        "--signer-public-key",
        "ed25519:7fvCiaE4NTmhexo8fDoa3CFNupL6mvJmNjL1hydN65fm",
        "--signer-private-key",
        "ed25519:VzeoRptTNGWeXwq3JNLdo8XqoBKvjSXyV5VxjvxPyzsiGmwo2Vu6LTiBujoQSXYjF8khQS5r3SSQK3xV8uomjv7",
        "send",
    ])
    .assert()
    .success()
    .stdout(predicates::str::contains("Deployment successful"));

    // Step 8: Clean up is handled automatically by `tempdir`
}