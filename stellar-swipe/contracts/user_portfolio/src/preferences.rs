//! User notification preferences (Issue #430).
//!
//! Stores per-user on-chain notification preferences so the frontend can
//! filter event subscriptions accordingly.

use crate::storage::DataKey;
use soroban_sdk::{contracttype, Address, Env};

/// Notification preferences for a user.
/// Default: all alerts enabled.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotificationPrefs {
    pub stop_loss_alerts: bool,
    pub take_profit_alerts: bool,
    pub signal_expiry_alerts: bool,
    /// Alert when a followed provider posts a new signal.
    pub new_signal_alert: bool,
    pub leaderboard_rank_change: bool,
}

impl NotificationPrefs {
    /// Returns the default preferences with all alerts enabled.
    pub fn default_prefs() -> Self {
        NotificationPrefs {
            stop_loss_alerts: true,
            take_profit_alerts: true,
            signal_expiry_alerts: true,
            new_signal_alert: true,
            leaderboard_rank_change: true,
        }
    }
}

/// Store notification preferences for `user`. Caller must be `user`.
pub fn set_notification_preferences(env: &Env, user: &Address, prefs: NotificationPrefs) {
    user.require_auth();
    env.storage()
        .persistent()
        .set(&DataKey::NotificationPrefs(user.clone()), &prefs);
}

/// Retrieve notification preferences for `user`.
/// Returns default (all enabled) if never set.
pub fn get_notification_preferences(env: &Env, user: &Address) -> NotificationPrefs {
    env.storage()
        .persistent()
        .get(&DataKey::NotificationPrefs(user.clone()))
        .unwrap_or_else(NotificationPrefs::default_prefs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{UserPortfolio, UserPortfolioClient};
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env};

    fn setup() -> (Env, Address, UserPortfolioClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let oracle = Address::generate(&env);
        #[allow(deprecated)]
        let contract_id = env.register_contract(None, UserPortfolio);
        let client = UserPortfolioClient::new(&env, &contract_id);
        client.initialize(&admin, &oracle);
        (env, contract_id, client)
    }

    #[test]
    fn default_preferences_all_enabled() {
        let (env, _, client) = setup();
        let user = Address::generate(&env);
        let prefs = client.get_notification_preferences(&user);
        assert!(prefs.stop_loss_alerts);
        assert!(prefs.take_profit_alerts);
        assert!(prefs.signal_expiry_alerts);
        assert!(prefs.new_signal_from_followed_provider);
        assert!(prefs.leaderboard_rank_change);
    }

    #[test]
    fn set_and_get_preferences() {
        let (env, _, client) = setup();
        let user = Address::generate(&env);
        let prefs = NotificationPrefs {
            stop_loss_alerts: true,
            take_profit_alerts: false,
            signal_expiry_alerts: true,
            new_signal_alert: false,
            leaderboard_rank_change: true,
        };
        client.set_notification_preferences(&user, &prefs);
        let stored = client.get_notification_preferences(&user);
        assert_eq!(stored.stop_loss_alerts, true);
        assert_eq!(stored.take_profit_alerts, false);
        assert_eq!(stored.signal_expiry_alerts, true);
        assert_eq!(stored.new_signal_alert, false);
        assert_eq!(stored.leaderboard_rank_change, true);
    }

    #[test]
    fn update_preferences() {
        let (env, _, client) = setup();
        let user = Address::generate(&env);
        // First set
        let prefs1 = NotificationPrefs {
            stop_loss_alerts: false,
            take_profit_alerts: false,
            signal_expiry_alerts: false,
            new_signal_alert: false,
            leaderboard_rank_change: false,
        };
        client.set_notification_preferences(&user, &prefs1);
        // Update
        let prefs2 = NotificationPrefs {
            stop_loss_alerts: true,
            take_profit_alerts: true,
            signal_expiry_alerts: false,
            new_signal_alert: true,
            leaderboard_rank_change: false,
        };
        client.set_notification_preferences(&user, &prefs2);
        let stored = client.get_notification_preferences(&user);
        assert_eq!(stored.stop_loss_alerts, true);
        assert_eq!(stored.take_profit_alerts, true);
        assert_eq!(stored.signal_expiry_alerts, false);
        assert_eq!(stored.new_signal_alert, true);
        assert_eq!(stored.leaderboard_rank_change, false);
    }
}
