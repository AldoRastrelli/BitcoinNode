use super::config_vars::ConfigVars;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{self, prelude::*};

/// This function parses a TOML string and returns a `HashMap<String, String>`.
/// # Errors
/// - This function returns an error if the TOML string is invalid.
/// - This function returns an error if the file is not found.
/// - This functions returns and error if the `config_var` is not present in the config file or invalid.
pub fn get_config_var(config_var: &ConfigVars) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(ConfigVars::CONFIG_PATH.descr)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config_var_str = parse_toml(&contents)
        .get(config_var.descr)
        .ok_or("Missing or invalid config_var")?
        .to_string();

    Ok(config_var_str)
}

/// This function returns the logger file location from the config file.
/// # Errors
/// - This function returns an error if the logger file location is not present in the config file.
pub fn get_logger_file_location() -> Result<String, Box<dyn Error>> {
    get_config_var(&ConfigVars::LOGGER_FILE_LOCATION)
}

/// This function returns the DNS from the config file.
/// # Errors
/// - This function returns an error if the DNS is not present in the config file.
pub fn get_dns() -> Result<String, Box<dyn Error>> {
    get_config_var(&ConfigVars::DNS)
}

pub fn get_server_seed() -> Result<String, Box<dyn Error>> {
    get_config_var(&ConfigVars::SERVER_SEED)
}

/// This function returns the Protocol Version from the config file.
/// # Errors
/// - This function returns an error if the Protocol Version does not exist in the config file.
pub fn get_protocol_version() -> Result<u32, Box<dyn Error>> {
    let mut file = File::open("config.toml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let binding = parse_toml(&contents);
    let protocol_version_string = binding
        .get("protocol_version")
        .ok_or("Missing or invalid protocol_version")?;

    let protocol_version = protocol_version_string.parse::<u32>().map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to parse protocol_version: {}", e),
        )
    })?;

    Ok(protocol_version)
}

/// This function returns the Protocol Version from the config file with i32 type.
/// # Errors
/// - This function returns an error if the Protocol Version does not exist in the config file.
pub fn get_protocol_version_i32() -> Result<i32, Box<dyn Error>> {
    match get_protocol_version() {
        Ok(protocol_version) => Ok(protocol_version as i32),
        Err(e) => Err(e),
    }
}

/// This function returns the project start date from the config file.
/// # Errors
/// - This function returns an error if the project start date is not present in the config file.
pub fn get_project_start_date() -> Result<String, Box<dyn Error>> {
    get_config_var(&ConfigVars::PROJECT_START_DATE)
}

/// This function returns the testnet port from the config file.
/// # Errors
/// - This function returns an error if the testnet port is not present in the config file.
pub fn get_testnet_port() -> Result<u16, Box<dyn Error>> {
    let testnet_port = get_config_var(&ConfigVars::TESTNET_PORT);

    match testnet_port {
        Ok(port) => {
            if let Ok(testnet_port_int) = port.parse::<u16>() {
                return Ok(testnet_port_int);
            }
            Err("Failed to parse testnet_port as integer".into())
        }
        Err(e) => Err(e),
    }
}

/// This function prints the configs from the config file.
/// # Errors
/// - This function returns an error if the `logger_file_location` is not present in the config file.
/// - This function returns an error if the `dns` is not present in the config file.
/// - This function returns an error if the `protocol_version` is not present in the config file.
/// - This function returns an error if the `project_start_date` is not present in the config file.
pub fn print_configs() -> Result<(), Box<dyn Error>> {
    let mut file = File::open("config.toml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let config = parse_toml(&contents);

    let _logger_file_location = config
        .get("logger_file_location")
        .ok_or("Missing or invalid logger_file_location")?
        .to_string();

    let _dns = config
        .get("dns")
        .ok_or("Missing or invalid dns")?
        .to_string();

    let _protocol_version = config
        .get("protocol_version")
        .ok_or("Missing or invalid protocol_version")?
        .to_string();

    let _project_start_date = config
        .get("project_start_date")
        .ok_or("Missing or invalid project_start_date")?
        .to_string();

    //   println!("Logger file location: {logger_file_location}");
    //   println!("DNS: {dns}");
    //   println!("Protocol Version: {protocol_version}");
    //   println!("Project Start Date: {project_start_date}");

    Ok(())
}
/// This function parses the config file and returns a HashMap of the key-value pairs.
fn parse_toml(contents: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    let mut current_section = "";

    for line in contents.lines() {
        // Skip comments and empty lines
        if line.trim().starts_with('#') || line.trim().is_empty() {
            continue;
        }

        // Section headers
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line.trim_start_matches('[').trim_end_matches(']');
            continue;
        }

        // Key-value pairs
        if let Some(pos) = line.find('=') {
            let key = line[..pos].trim().to_owned();
            let value = line[pos + 1..].trim().to_owned().replace('"', "");

            if !key.is_empty() && !value.is_empty() {
                let key = if current_section.is_empty() {
                    key
                } else {
                    format!("{current_section}.{key}")
                };
                result.insert(key, value);
            }
        }
    }

    result
}

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_get_dns() {
        let dns = get_dns();
        assert!(dns.is_ok());
    }

    #[test]
    fn test_get_testnet_port() {
        let test_net = get_testnet_port();
        assert!(test_net.is_ok());
    }

    #[test]
    fn test_print_configs() {
        let configs = print_configs();
        assert!(configs.is_ok());
    }

    #[test]
    fn test_get_logger_file_location() {
        let logger_file_location = get_logger_file_location();
        assert!(logger_file_location.is_ok());
    }

    #[test]
    fn test_get_protocol_version() {
        let protocol_version = get_protocol_version();
        assert!(protocol_version.is_ok());
    }

    #[test]
    fn test_get_project_start_date() {
        let project_start_date = get_project_start_date();
        assert!(project_start_date.is_ok());
    }

    #[test]
    fn test_get_config_var() {
        let config_var = get_config_var(&ConfigVars::DNS);
        assert!(config_var.is_ok());
    }

    #[test]
    fn test_get_config_var_invalid() {
        let config_var = get_config_var(&ConfigVars::INVALID);
        assert!(config_var.is_err());
    }
}
