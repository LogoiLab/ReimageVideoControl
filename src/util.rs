use std::process::Command;
use super::serialport::SerialPort;

pub fn get_color(percent: f64) -> char {
    let mut color: char = 'g';
    if percent > 0.70 && percent < 0.89 {
        color = 'y';
    } else if percent > 0.90 {
        color = 'r';
    }
    return color;
}

pub fn get_partition_usage(partition_path: String) -> f64 {
    let df_raw = Command::new("df")
        .arg(partition_path)
        .arg("--no-sync")
        .arg("--output=pcent")
        .output()
        .expect("Failed to execute `df` process, please make sure you have `df` installed");
    let df_string = String::from_utf8(df_raw.stdout).unwrap();
    let df_split: Vec<&str> = df_string.split('\n').collect();
    let mut percent: String;
    if df_split.len() > 1 {
        percent = String::from(df_split[1]).replace('%', "");
        percent = String::from(percent.trim());
        if percent.len() == 1 {
            percent.insert(0, '0');
        }
    } else {
        percent = String::from("00");
    }
        percent.insert(percent.len() - 2, '.');
    if percent.starts_with('.') {
        percent.insert(0, '0');
    }
    return f64::trunc((match percent.parse::<f64>() {
        Ok(o) => o,
        Err(_) => 0.0
    }) * 100.0) / 100.0;
}

pub fn clear(port: &mut Box<dyn SerialPort>) {
    port.write("c\n".as_bytes()).expect("failed to send clear screen command");
}
