use clap::Subcommand;
use log::info;

use crate::{
    interactive::subscription::{mihomo_config_path, refresh_subscription},
    interactive::Flags,
    Result,
};

#[derive(Subcommand, Debug)]
#[clap(about = "Manage subscription")]
pub enum SubscriptionSubcommand {
    #[clap(about = "Refresh subscription from remote URL")]
    Refresh,
}

impl SubscriptionSubcommand {
    pub fn handle(&self, flags: &Flags) -> Result<()> {
        let config = flags.get_config()?;

        match self {
            Self::Refresh => {
                let sub_config = config
                    .tui
                    .subscription
                    .as_ref()
                    .ok_or_else(|| {
                        crate::Error::custom(
                            "No subscription configured in config.ron (tui.subscription)".into(),
                        )
                    })?;

                refresh_subscription(sub_config).map_err(|e| {
                    crate::Error::custom(e.to_string())
                })?;

                info!("Subscription refreshed successfully");

                // Optionally reload clash configs
                if let Ok(clash) = flags.connect_server_from_config() {
                    let path = mihomo_config_path();
                    match clash.reload_configs(false, &path.to_string_lossy()) {
                        Ok(()) => info!("Clash configs reloaded"),
                        Err(e) => log::warn!("Failed to reload clash configs: {:?}", e),
                    }
                }
            }
        }
        Ok(())
    }
}
