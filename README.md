# BOS CLI

Command line utility helps to develop components for [NEAR Blockchain Operating System](https://near.org/blog/near-announces-the-blockchain-operating-system/) by allowing developers to use standard developer tools like their best code editor and standard tools for source code version control, and then deploy their components to SocialDB in one command.

Currently, only two groups of commands are implemented:

- `components`  -   Working with components (Download, Deploy, etc.)
- `socialdb`    -   SocialDb management

### components  -   Working with components (Download, Deploy, etc.)

- `deploy` allows you to upload/publish components from your local `./src` folder to near.social account.
- `download` allows you to download the existing components from any near.social account to the local `./src` folder.
- `delete` allows you to delete the existing components from any near.social account.

### socialdb    -   SocialDb management

#### prepaid-storage   -   Storage management: deposit, withdrawal, balance review

- `view-balance` allows you to view the storage balance for an account.
- `deposit` allows you to make a storage deposit for the account.
- `withdraw` allows you to make a withdraw a deposit from storage for an account ID.

#### permissions       -   Granting access permissions to a different account

- `grant-write-access` allows grant access to the access key to call a function or another account.

More commands are still on the way, see the [issues tracker](https://github.com/FroVolod/bos-cli-rs/issues) and propose more features there.

## Install

### From Binaries

The [release page](https://github.com/FroVolod/bos-cli-rs/releases) includes precompiled binaries for Linux, macOS and Windows.

### From Source

With Rust's package manager cargo, you can install `bos` via:

```
cargo install --git https://github.com/FroVolod/bos-cli-rs
```

### GitHub Actions

#### Reusable Workflow

This repo contains a reusable workflow which you can directly leverage from your component repository

1. Prepare access key that will be used for components deployment.

   It is recommended to use a dedicated function-call-only access key, so you need to:

   1.1. Add a new access key to your account. Here is [near CLI](https://near.cli.rs) command to do that:

   ```bash
   near account add-key "ACCOUNT_ID" grant-function-call-access --allowance '1 NEAR' --receiver-account-id social.near --method-names 'set' autogenerate-new-keypair print-to-terminal network-config mainnet
   ```
   1.2. Grant write permission to the key (replace `PUBLIC_KEY` with the one you added to the account on the previous step, and `ACCOUNT_ID` with the account id where you want to deploy BOS components):

   ```bash
   near contract call-function as-transaction social.near grant_write_permission json-args '{"public_key": "PUBLIC_KEY", "keys": ["ACCOUNT_ID/widget"]}' prepaid-gas '100.000 TeraGas' attached-deposit '1 NEAR' sign-as "ACCOUNT_ID" network-config mainnet
   ```

   Note: the attached deposit is going to be used to cover the storage costs associated with the data you store on BOS, 1 NEAR is enough to store 100kb of data (components code, metadata, etc).
2. In your repo, go to _Settings > Secrets and Variables > Actions_ and create a new repository secret named `SIGNER_PRIVATE_KEY` with the private key in `ed25519:<private_key>` format (if you followed (1.1), it is be printed in your terminal)
3. Create a file at `.github/workflows/deploy-mainnet.yml` in your component repo with the following contents.
   See the [workflow definition](./github/workflows/deploy-mainnet.yml) for explanations of the inputs

    ```yml
    name: Deploy Components to Mainnet
    on:
      push:
        branches: [main]
    jobs:
      deploy-mainnet:
        uses: FroVolod/bos-cli-rs/.github/workflows/deploy-mainnet.yml@master
        with:
          deploy-account-address: <FILL>
          signer-account-address: <FILL>
          signer-public-key: <FILL>
        secrets:
          SIGNER_PRIVATE_KEY: ${{ secrets.SIGNER_PRIVATE_KEY }}
    ```

4. Commit and push the workflow
5. On changes to the `main` branch, updated components in `src` will be deployed!

#### Custom Workflow

Copy the contents of `.github/workflows/deploy-mainnet.yml` to your repo as a starting point
