use super::serde_derive::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub serial_port: String,
    pub hostname_kernel: bool,
    pub uptime: bool,
    pub cpu_utilization: bool,
    pub memory_utilization: bool,
    pub network_utilization: NetUtil,
    pub storage_utilization: StorageUtil
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NetUtil {
    pub enabled: bool,
    pub network: Option<Vec<NetConf>>
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NetConf {
    pub interface_name: String,
    pub top_speed_bps: u64
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StorageUtil {
    pub enabled: bool,
    pub partition: Option<Vec<PartConf>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PartConf {
    pub partition_name: String,
    pub partition_path: String,
}

/* Example config generation code.
let default_config: Config = Config{
    serial_port: "/dev/ttyUSB0".to_string(),
    hostname_kernel: true,
    uptime: true,
    cpu_utilization: true,
    memory_utilization: true,
    network_utilization: NetUtil {
        enabled: true,
        network: Some(vec!(
                NetConf{
                    interface_name: "eno1".to_string(),
                    top_speed_bps: 1000000000
                }
            ))
        },
    storage_utilization: StorageUtil{
        enabled: true,
        partition: Some(vec!(
            PartConf {
                partition_name: "Internal RAID".to_string(),
                partition_path: "/dev/sdz".to_string()
            },
            PartConf {
                partition_name: "External RAID".to_string(),
                partition_path: "/dev/sdb".to_string()
            }
        ))
    }
};
 */

impl Config {
    pub fn parse_from_file(path: &str) -> Config {
        match std::fs::read_to_string(path) {
            Ok(file_contents) => match toml::from_str(file_contents.as_str()) {
                Ok(config) => return config,
                Err(e) => {
                    eprintln!("Fatal, failed to parse configuration file: {}", e);
                    std::process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("Failed to read config file: {}", e);
                std::process::exit(1);
            }
        }
    }
}
