# near-social CLI

Command line utility helps to develop widgets for [near.social](https://near.social) by allowing developers to use standard developer tools like their best code editor and standard tools for source code version control, and then deploy their widgets to SocialDB in one command.

There are currently only two commands implemented:
* `deploy` allows you to upload/publish widgets from your local `./src` folder to near.social account.
* `download` allows you to download the existing widgets from any near.social account to the local `./src` folder.

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

You can automate widgets deployment being done on every commit to `main` branch (or chose your own condition) with the following script (put it into `.github/workflows/release.yml` of your project):

```yml
name: Release
on:
  push:
    branches: [main]
jobs:
  deploy-widgets:
    runs-on: ubuntu-latest
    name: Deploy widgets to near.social (mainnet)
    env:
      NEAR_SOCIAL_ACCOUNT_ID: ${{ vars.NEAR_SOCIAL_ACCOUNT_ID }}
      NEAR_SOCIAL_ACCOUNT_PUBLIC_KEY: ${{ vars.NEAR_SOCIAL_ACCOUNT_PUBLIC_KEY }}
      NEAR_SOCIAL_ACCOUNT_PRIVATE_KEY: ${{ secrets.NEAR_SOCIAL_ACCOUNT_PRIVATE_KEY }}

    steps:
    - name: Checkout repository
      uses: actions/checkout@v2

    - name: Install near-social CLI
      run: |
        curl --proto '=https' --tlsv1.2 -L -sSf https://github.com/FroVolod/near-social/releases/download/v0.2.3/installer.sh | sh

    - name: Deploy widgets
      run: |
        near-social deploy "$NEAR_SOCIAL_ACCOUNT_ID" sign-as "$NEAR_SOCIAL_ACCOUNT_ID" network-config mainnet sign-with-plaintext-private-key --signer-public-key "$NEAR_SOCIAL_ACCOUNT_PUBLIC_KEY" --signer-private-key "$NEAR_SOCIAL_ACCOUNT_PRIVATE_KEY" send
```
