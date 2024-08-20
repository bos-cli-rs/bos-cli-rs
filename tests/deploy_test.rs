// tests/deploy_test.rs

use assert_cmd::Command;
use httpmock::MockServer;
use httpmock::prelude::*;
use std::env;
use std::fs;
use tempfile::tempdir;
use serde_json::json;

#[test]
fn test_bos_components_deploy() {
    // Step 1: Set up a temporary directory for components
    let temp_dir = tempdir().unwrap();
    let src_dir = temp_dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    // Create a mock component file
    let component_path = src_dir.join("example_component.jsx");
    fs::write(&component_path, "console.log('Hello, world!');").unwrap();

    // Step 2: Start a mock server to simulate the NEAR RPC server
    let server = MockServer::start();

    // Step 3: Mock the response for fetching the access key with nonce
    let access_key_mock = server.mock(|when, then| {
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

    // Step 4: Mock other necessary RPC calls as needed

    // Step 5: Change the current directory to the temporary directory
    env::set_current_dir(&temp_dir).unwrap();

    // Step 6: Run the CLI command as a subprocess
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
    .env("NEAR_RPC_URL", &server.url("/")) // Use the mock server's URL
    .assert()
    .success()
    .stdout(predicates::str::contains("Deployment successful"));

    // Step 7: Clean up is handled automatically by `tempdir`
}