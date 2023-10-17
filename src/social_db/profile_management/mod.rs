use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod view_profile;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
pub struct ManageProfile {
    #[interactive_clap(subcommand)]
    actions: Actions,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// What do you want to do with the profile?
pub enum Actions {
    #[strum_discriminants(strum(message = "view-profile    - View profile for an account"))]
    /// View profile for an account
    ViewProfile(self::view_profile::Account),
    #[strum_discriminants(strum(message = "update-profile  - Update profile for the account"))]
    /// Update profile for the account
    UpdateProfile(near_cli_rs::commands::account::update_social_profile::UpdateSocialProfile),
}
