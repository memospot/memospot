use std::collections::HashMap;

use figment::{providers::Serialized, util::map};

use crate::config::CONFIG_PROFILE;

pub trait MigrationExt {
    fn migrate(self) -> figment::Figment;
    fn v_0_1_7_to_v1_0_0(self) -> figment::Figment;
}

impl MigrationExt for figment::Figment {
    /// Migrate previous configuration format to the current one.
    fn migrate(self) -> figment::Figment {
        self.v_0_1_7_to_v1_0_0()
    }

    /// Upgrade from Memospot v0.1.7.
    ///
    /// Overrides `memos.env` with the new structure format, preventing errors if `memos.env=null`.
    fn v_0_1_7_to_v1_0_0(self) -> figment::Figment {
        if let Ok(previous_vars) = self.extract_inner::<HashMap<String, String>>("memos.env") {
            let memos_env_enabled = map!["memos"  => map!["env" => map!["enabled" => true]]];
            let memos_env_vars = map!["memos"  => map!["env" => map!["vars" => previous_vars]]];
            self.merge(Serialized::from(memos_env_enabled, CONFIG_PROFILE))
                .merge(Serialized::from(memos_env_vars, CONFIG_PROFILE))
        } else {
            let memos_env_enabled = map!["memos"  => map!["env" => map!["enabled" => false]]];
            self.merge(Serialized::from(memos_env_enabled, CONFIG_PROFILE))
        }
    }
}
