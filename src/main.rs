extern crate cpu_monitor;
extern crate serde;
extern crate serde_derive;
extern crate serialport;
extern crate toml;

use std::io::Write;
use std::time::Duration;

use cpu_monitor::CpuInstant;
use serde_derive::{Serialize, Deserialize};
use sysinfo::{NetworkExt, System, SystemExt};

pub mod util;

use util::*;

#[derive(Serialize, Deserialize)]
pub struct Config {
    serial_port: String,
    hostname_kernel: bool,
    uptime: bool,
    cpu_utilization: bool,
    memory_utilization: bool,
    network_utilization: NetUtil,
    storage_utilization: StorageUtil
}

#[derive(Serialize, Deserialize)]
pub struct NetUtil {
    enabled: bool,
    network: Option<Vec<NetConf>>
}

#[derive(Serialize, Deserialize)]
pub struct NetConf {
    interface_name: String,
    top_speed_mbps: u64
}

#[derive(Serialize, Deserialize)]
pub struct StorageUtil {
    enabled: bool,
    partition: Option<Vec<PartConf>>,
}

#[derive(Serialize, Deserialize)]
pub struct PartConf {
    partition_name: String,
    partition_path: String,
}

fn main() {
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
                    top_speed_mbps: 1000000000
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
    println!("{}", toml::to_string(&default_config).unwrap());
    let ports = serialport::available_ports().expect("No ports found!");
    for p in ports {
        println!("{}", p.port_name);
    }
    let port_settings = serialport::SerialPortSettings {
        baud_rate: 14400,
        data_bits: serialport::DataBits::Eight,
        flow_control: serialport::FlowControl::None,
        parity: serialport::Parity::None,
        stop_bits: serialport::StopBits::One,
        timeout: Duration::new(65535, 0)
    };
    let mut port = match serialport::open_with_settings("/dev/ttyUSB0", &port_settings) {
        Ok(o) => o,
        Err(_) => {
            eprintln!("Failed to connect to serial port: /dev/ttyUSB0");
            std::process::exit(1);
        }
    };
    let mut sys = System::new_all();
    loop {
        sys.refresh_all();
        std::thread::sleep(Duration::from_millis(100));
        clear(&mut port);
        std::thread::sleep(Duration::from_millis(100));
        port.write(format!("t g {} {}\n", sys.host_name().unwrap_or("".to_string()), sys.kernel_version().unwrap_or("".to_string())).as_bytes()).unwrap();
        std::thread::sleep(Duration::from_secs(10));
        clear(&mut port);
        let mut time: u64 = 0;
        clear(&mut port);
        std::thread::sleep(Duration::from_millis(100));
        while time < 10 {
            let uptime = sys.uptime();
            let seconds = uptime % 60;
            let minutes = uptime / 60;
            let hours = minutes / 60;
            let days = hours / 24;
            let minutes_remainder = (uptime / 60) - (hours * 60);
            let hours_remainder = (minutes / 60) - (days * 24);
            port.write(format!("t g Uptime: {}d {}h {}m {}s\n", days, hours_remainder, minutes_remainder, seconds).as_bytes()).unwrap();
            std::thread::sleep(Duration::from_secs(1));
            time = time + 1;
        }
        clear(&mut port);
        std::thread::sleep(Duration::from_millis(100));
        port.write("t g CPU Usage\n".as_bytes()).unwrap();
        time = 0;
        while time < 100 {
            let start = CpuInstant::now().unwrap();
            std::thread::sleep(Duration::from_millis(100));
            let end = CpuInstant::now().unwrap();
            let duration = end - start;
            let percent = f64::trunc(duration.non_idle() * 100.0) / 100.0;

            let output = format!("p {} {}\n", get_color(percent), percent);
            port.write(output.as_bytes()).expect("Write failed!");
            time = time + 1;
        }
        time = 0;
        clear(&mut port);
        std::thread::sleep(Duration::from_millis(100));
        while time < 20 {
            sys.refresh_memory();
            let mem_total_mb = sys.total_memory() / 1000;
            let mem_used_mb = sys.used_memory() / 1000;
            let mut mem_percent: f64 = sys.used_memory() as f64 / sys.total_memory() as f64;
            mem_percent = f64::trunc(mem_percent * 100.0) / 100.0;
            port.write(format!("p {} {}\n", get_color(mem_percent), mem_percent).as_bytes()).unwrap();
            std::thread::sleep(Duration::from_millis(100));
            port.write(format!("t {} Memory {}/{}MB\n", get_color(mem_percent), mem_used_mb, mem_total_mb).as_bytes()).unwrap();
            std::thread::sleep(Duration::from_millis(400));
            time = time + 1;
        }
        port.write("p g 0.0\n".as_bytes()).unwrap();
        let mut bwload: u64 = 0;
        sys.refresh_networks();
        let mut _a: u64;
        let mut _b: u64;
        for (_, data) in sys.networks() {
            _a = data.received();
            _b = data.transmitted();
        }
        //port.write("t g Network:\n".as_bytes()).unwrap();
        std::thread::sleep(Duration::from_millis(100));
        time = 0;
        while time < 10 {
            sys.refresh_networks();
            std::thread::sleep(Duration::from_millis(900));
            for (interface_name, data) in sys.networks() {
                if interface_name.eq("eno1") {
                    bwload = data.received() * 8;
                    bwload += data.transmitted() * 8;
                }
            }
            let bwload_percent = f64::trunc((bwload as f64 / 1000000000.0) * 100.0) / 100.0;
            port.write(format!("p {} {}\n", get_color(bwload_percent), bwload_percent).as_bytes()).unwrap();
            std::thread::sleep(Duration::from_millis(100));
            port.write(format!("t {} Network: {}Mbps\n", get_color(bwload_percent), f64::trunc((bwload as f64 / 1000000.0) * 100.0) / 100.0).as_bytes()).unwrap();
            time = time + 1;
        }
        std::thread::sleep(Duration::from_millis(100));
        clear(&mut port);
        std::thread::sleep(Duration::from_millis(100));
        let part_one = get_partition_usage(String::from("/dev/sdz"));
        port.write(format!("p {} {}\n", get_color(part_one), part_one).as_bytes()).unwrap();
        std::thread::sleep(Duration::from_millis(100));
        port.write(format!("t {} Internal RAID\n", get_color(part_one)).as_bytes()).unwrap();
        std::thread::sleep(Duration::from_secs(10));
        clear(&mut port);
        std::thread::sleep(Duration::from_millis(100));
        let part_two = get_partition_usage(String::from("/dev/sdb"));
        port.write(format!("p {} {}\n", get_color(part_two), part_two).as_bytes()).unwrap();
        std::thread::sleep(Duration::from_millis(100));
        port.write(format!("t {} External RAID\n", get_color(part_two)).as_bytes()).unwrap();
        std::thread::sleep(Duration::from_secs(10));
    }
}
