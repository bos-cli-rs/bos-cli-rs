# near-social CLI

This command line utility helps to develop widgets for [near.social](https://near.social) using standard tools like your best code editor, use standard tools for source code version control, etc.

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
