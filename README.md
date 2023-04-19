# near-social CLI

Command line utility helps to develop widgets for [near.social](https://near.social) by allowing developers to use standard developer tools like their best code editor and standard tools for source code version control, and then deploy their widgets to SocialDB in one command.

There are currently only two commands implemented:

- `deploy` allows you to upload/publish widgets from your local `./src` folder to near.social account.
- `download` allows you to download the existing widgets from any near.social account to the local `./src` folder.

This tools is in its early stage, so there are some known limitations around storage deposit.
More commands are still on the way, see the [issues tracker](https://github.com/FroVolod/near-social/issues) and propose more features there.
Yet, NEAR GigsBoard uses this CLI in production for Continuous Delivery (CD) setup, check it out [here](https://github.com/near/devgigsboard-widgets/blob/69fb12cf2fb62d14db6911661bac77cdc969a8b4/.github/workflows/release.yml).

Watch an early intro screencast tour [here](https://www.loom.com/share/8b6c3509eb61498b8bffbe65a625616d).

## Install

### From Binaries

The [release page](https://github.com/FroVolod/near-social/releases) includes precompiled binaries for Linux, macOS and Windows.

### From Source

With Rust's package manager cargo, you can install near-social via:

```
cargo install --git https://github.com/FroVolod/near-social
```

### GitHub Actions

#### Reusable Workflow

This repo contains a reusable workflow which you can directly leverage from your component repository

1. TODO: generate public and private key
2. In your repo, go to _Settings > Secrets and Variables > Actions_ and create a new repository secret named `SIGNER_PRIVATE_KEY` with the generated private key in `ed25519:<private_key>` format
3. Create a file at `.github/workflows/deploy-mainnet.yml` in your component repo with the following contents.
   See the [workflow definition](./github/workflows/deploy-mainnet.yml) for explanations of the inputs

```yml
name: Deploy Components to Mainnet
on:
  push:
    branches: [main]
jobs:
  deploy-mainnet:
    uses: FroVolod/near-social/.github/workflows/deploy-mainnet.yml@main
    with:
      cli-version: <FILL>
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
