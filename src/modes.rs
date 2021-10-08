use std::time::Duration;

use super::cpu_monitor::CpuInstant;
use super::serialport::SerialPort;
use super::sysinfo::{NetworkExt, System, SystemExt};

use super::config::*;
use super::util::*;

pub fn do_hostname_kernel(sys: &mut System, port: &mut Box<dyn SerialPort>) {
    sys.refresh_all();
    std::thread::sleep(Duration::from_millis(100));
    clear(port);
    std::thread::sleep(Duration::from_millis(100));
    port.write(format!("t g {} {}\n", sys.host_name().unwrap_or("".to_string()), sys.kernel_version().unwrap_or("".to_string())).as_bytes()).unwrap();
    std::thread::sleep(Duration::from_secs(10));
}

pub fn do_uptime(sys: &mut System, port: &mut Box<dyn SerialPort>) {
    sys.refresh_all();
    std::thread::sleep(Duration::from_millis(100));
    clear(port);
    std::thread::sleep(Duration::from_millis(100));
    let mut time: u64 = 0;
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
}

pub fn do_cpu_utilization(port: &mut Box<dyn SerialPort>) {
    clear(port);
    std::thread::sleep(Duration::from_millis(100));
    port.write("t g CPU Usage\n".as_bytes()).unwrap();
    let mut time = 0;
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
}

pub fn do_memory_utilization(sys: &mut System, port: &mut Box<dyn SerialPort>) {
    let mut time = 0;
    clear(port);
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
}

pub fn do_network_utilization(sys: &mut System, port: &mut Box<dyn SerialPort>, confs: Vec<NetConf>) {
    for conf in confs {
        clear(port);
        std::thread::sleep(Duration::from_millis(100));
        port.write(format!("t g {}\n", conf.interface_name).as_bytes()).unwrap();
        let mut bwload: u64 = 0;
        sys.refresh_networks();
        let mut _a: u64;
        let mut _b: u64;
        for (_, data) in sys.networks() {
            _a = data.received();
            _b = data.transmitted();
        }
        std::thread::sleep(Duration::from_millis(100));
        let mut time = 0;
        while time < 10 {
            sys.refresh_networks();
            std::thread::sleep(Duration::from_millis(900));
            for (interface_name, data) in sys.networks() {
                if interface_name.eq(conf.interface_name.as_str()) {
                    bwload = data.received() * 8;
                    bwload += data.transmitted() * 8;
                }
            }
            let bwload_percent = f64::trunc((bwload as f64 / conf.top_speed_bps as f64) * 100.0) / 100.0;
            port.write(format!("p {} {}\n", get_color(bwload_percent), bwload_percent).as_bytes()).unwrap();
            std::thread::sleep(Duration::from_millis(100));
            port.write(format!("t {} {}: {}Mbps\n", get_color(bwload_percent), conf.interface_name, f64::trunc((bwload as f64 / 1000000.0) * 100.0) / 100.0).as_bytes()).unwrap();
            time = time + 1;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

pub fn do_partition_utilization(port: &mut Box<dyn SerialPort>, confs: Vec<PartConf>) {
    for conf in confs {
        clear(port);
        std::thread::sleep(Duration::from_millis(100));
        let part_one = get_partition_usage(conf.partition_path);
        port.write(format!("p {} {}\n", get_color(part_one), part_one).as_bytes()).unwrap();
        std::thread::sleep(Duration::from_millis(100));
        port.write(format!("t {} {}\n", get_color(part_one), conf.partition_name).as_bytes()).unwrap();
        std::thread::sleep(Duration::from_secs(10));
    }
}
