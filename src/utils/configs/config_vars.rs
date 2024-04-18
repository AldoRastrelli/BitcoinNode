/// This module contains the configuration variables for the project.
pub struct ConfigVars<'a> {
    pub descr: &'a str,
}

impl ConfigVars<'_> {
    /// The path to the configuration file
    pub const CONFIG_PATH: Self = Self {
        descr: "config.toml",
    };
    /// The location of the logger file
    pub const LOGGER_FILE_LOCATION: Self = Self {
        descr: "logger_file_location",
    };
    /// The DNS url
    pub const DNS: Self = Self { descr: "dns" };
    /// The Protocol Version
    pub const PROTOCOL_VERSION: Self = Self {
        descr: "protocol_version",
    };
    /// The Project's start date
    pub const PROJECT_START_DATE: Self = Self {
        descr: "project_start_date",
    };
    /// The Testnet Port
    pub const TESTNET_PORT: Self = Self {
        descr: "testnet_port",
    };

    pub const SERVER_SEED: Self = Self {
        descr: "server_seed",
    };

    /// For testing purposes only
    pub const INVALID: Self = Self { descr: "invalid" };
}

#[cfg(test)]

mod config_vars_tests {
    use super::*;

    #[test]
    fn config_path_description() {
        assert_eq!(ConfigVars::CONFIG_PATH.descr, "config.toml");
    }

    #[test]
    fn logger_file_location_description() {
        assert_eq!(
            ConfigVars::LOGGER_FILE_LOCATION.descr,
            "logger_file_location"
        );
    }

    #[test]
    fn dns_description() {
        assert_eq!(ConfigVars::DNS.descr, "dns");
    }

    #[test]
    fn protocol_version_description() {
        assert_eq!(ConfigVars::PROTOCOL_VERSION.descr, "protocol_version");
    }

    #[test]
    fn project_start_date_description() {
        assert_eq!(ConfigVars::PROJECT_START_DATE.descr, "project_start_date");
    }

    #[test]
    fn testnet_port_description() {
        assert_eq!(ConfigVars::TESTNET_PORT.descr, "testnet_port");
    }

    #[test]
    fn invalid_description() {
        assert_eq!(ConfigVars::INVALID.descr, "invalid");
    }
}
