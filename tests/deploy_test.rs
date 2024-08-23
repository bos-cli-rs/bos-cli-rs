mod test_util;
use assert_cmd::Command;
use dirs;
use httpmock::prelude::HttpMockRequest;
use std::env;
use std::fs;
use test_util::*;

static COMPONENT_CONTENT: &str = "return <>hello</>";

#[test]
fn test_bos_components_deploy_with_mocked_rpc() {
    // Start a mock server to simulate the NEAR RPC server
    let mut server = setup_mock_server();

    // Mock the `broadcast_tx_commit` RPC call

    let broadcast_tx_commit_matcher = move |req: &HttpMockRequest| {
        if let Some(body) = &req.body {
            return match_broadcast_tx_commit_for_component_content(body, COMPONENT_CONTENT);
        }
        false
    };

    server = mock_broadcast_tx_commit(server, COMPONENT_CONTENT, broadcast_tx_commit_matcher);
    server = mock_unmatched(server);

    // Locate the existing config directory
    let config_dir = dirs::config_dir().unwrap().join("near-cli");

    // Backup and create new config.toml
    let backup_path = setup_config(&config_dir, &server.url("/"));

    // Set up a temporary directory for components
    let temp_dir = setup_temp_dir();

    // Create a mock component file in the temp directory
    let component_path = temp_dir.path().join("src").join("example_component.jsx");
    fs::write(&component_path, COMPONENT_CONTENT).unwrap();

    // Change the current directory to the temporary directory for components
    env::set_current_dir(&temp_dir).unwrap();

    // Run the CLI command as a subprocess
    let mut cmd = Command::cargo_bin("bos").unwrap();

    cmd.args(&[
        "components",
        "deploy",
        "test.near",
        "sign-as",
        "test.near",
        "network-config",
        "mainnet", // Use the mock network we added
        "sign-with-plaintext-private-key",
        "--signer-public-key",
        "ed25519:7fvCiaE4NTmhexo8fDoa3CFNupL6mvJmNjL1hydN65fm",
        "--signer-private-key",
        "ed25519:VzeoRptTNGWeXwq3JNLdo8XqoBKvjSXyV5VxjvxPyzsiGmwo2Vu6LTiBujoQSXYjF8khQS5r3SSQK3xV8uomjv7",
        "send",
    ])
    .assert()
    .success()
    .stdout(predicates::str::contains("components were successfully deployed"));

    // Restore the original config.toml if it existed
    restore_config(&config_dir, backup_path);

    // Clean up the temp directory is handled automatically by `tempdir`
}
