extern crate cpu_monitor;
extern crate serde;
extern crate serde_derive;
extern crate serialport;
extern crate sysinfo;
extern crate toml;

use std::time::Duration;

use sysinfo::{System, SystemExt};

pub mod config;
pub mod modes;
pub mod util;

use config::*;
use modes::*;

fn main() {
    let incoming_config = Config::parse_from_file("./rivctrl_conf.toml");
    let ports = serialport::available_ports().expect("No ports found!");
    print!("Available serial ports:");
    for p in ports {
        print!(" {}", p.port_name);
    }
    println!("\nUsing: {}", incoming_config.serial_port);

    let port_settings = serialport::SerialPortSettings {
        baud_rate: 14400,
        data_bits: serialport::DataBits::Eight,
        flow_control: serialport::FlowControl::None,
        parity: serialport::Parity::None,
        stop_bits: serialport::StopBits::One,
        timeout: Duration::new(65535, 0)
    };
    let mut port = match serialport::open_with_settings(incoming_config.serial_port.as_str(), &port_settings) {
        Ok(o) => o,
        Err(_) => {
            eprintln!("Failed to connect to serial port: {}", incoming_config.serial_port);
            std::process::exit(1);
        }
    };
    let mut sys = System::new_all();
    loop {
        let config = incoming_config.clone();
        if config.hostname_kernel {
            do_hostname_kernel(&mut sys, &mut port);
        }
        if config.uptime {
            do_uptime(&mut sys, &mut port);
        }
        if config.cpu_utilization {
            do_cpu_utilization(&mut port);
        }
        if config.memory_utilization {
            do_memory_utilization(&mut sys, &mut port);
        }
        if config.network_utilization.enabled {
            match config.network_utilization.network {
                Some(s) => do_network_utilization(&mut sys, &mut port, s),
                None => (),
            }
        }
        if config.storage_utilization.enabled {
            match config.storage_utilization.partition {
                Some(s) => do_partition_utilization(&mut port, s),
                None => (),
            }
        }
    }
}
