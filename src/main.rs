use ncurses::*;
use regex::Regex;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

fn setup_ncurses() {
    initscr(); // Initialize the ncurses library
    cbreak(); // Disable line buffering
    noecho(); // Disable automatic echoing of typed characters
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE); // Hide the cursor
    start_color(); // Enable color support

    // Define colors if needed
    init_pair(1, COLOR_WHITE, COLOR_BLACK);
    init_pair(2, COLOR_YELLOW, COLOR_BLACK);
}

fn update_hardware_data(fan_rpm: f64, cpu_temp: f64, gpu_temp: f64) {
    erase(); // Clear the entire screen

    let fan_str = format!("Fan: {:.2} rpm", fan_rpm);
    let cpu_temp_str = format!("CPU die temperature: {:.2} C", cpu_temp);
    let gpu_temp_str = format!("GPU die temperature: {:.2} C", gpu_temp);

    // Display the hardware data at specific positions
    mvprintw(0, 0, "---------------------------");
    mvprintw(1, 0, &fan_str);
    mvprintw(2, 0, &cpu_temp_str);
    mvprintw(3, 0, &gpu_temp_str);

    refresh(); // Refresh the screen
}

fn main() {
    setup_ncurses();

    let command = Command::new("sudo")
        .arg("powermetrics")
        .arg("--samplers")
        .arg("smc")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute powermetrics command");

    let stdout = command.stdout.expect("Failed to capture command output");
    let reader = BufReader::new(stdout);

    let fan_re = Regex::new(r"Fan: ([0-9.]+) rpm").unwrap();
    let cpu_temp_re = Regex::new(r"CPU die temperature: ([0-9.]+) C").unwrap();
    let gpu_temp_re = Regex::new(r"GPU die temperature: ([0-9.]+) C").unwrap();

    let mut fan_rpm: f64 = 0.0;
    let mut cpu_temp: f64 = 0.0;
    let mut gpu_temp: f64 = 0.0;
    let mut found_section = false;

    for line in reader.lines() {
        if let Ok(line) = line {
            if line.contains("**** SMC sensors ****") {
                found_section = true;
            } else if line.contains("****") && found_section {
                break; // Reached end of relevant section
            } else if found_section {
                if let Some(captures) = fan_re.captures(&line) {
                    fan_rpm = captures[1].parse::<f64>().unwrap();
                } else if let Some(captures) = cpu_temp_re.captures(&line) {
                    cpu_temp = captures[1].parse::<f64>().unwrap();
                } else if let Some(captures) = gpu_temp_re.captures(&line) {
                    gpu_temp = captures[1].parse::<f64>().unwrap();
                }
                if fan_rpm != 0.0 && cpu_temp != 0.0 && gpu_temp != 0.0 {
                    update_hardware_data(fan_rpm, cpu_temp, gpu_temp);
                    fan_rpm = 0.0;
                    cpu_temp = 0.0;
                    gpu_temp = 0.0;
                }
            }
        } else if let Err(err) = line {
            eprintln!("Error reading line: {}", err);
        }
    }

    // Sleep for some time to observe the changes
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Restore terminal settings and clean up ncurses
    endwin();
}
