use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

fn read_lines<P>(filename: P) -> io::Result<Vec<String>>
where
    P: AsRef<Path>,
{
    let file = fs::File::open(filename)?;
    let reader = BufReader::new(file);
    reader.lines().collect()
}

fn write_lines<P>(filename: P, lines: &[String]) -> io::Result<()>
where
    P: AsRef<Path>,
{
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(filename)?;

    for line in lines {
        writeln!(file, "{}", line)?;
    }
    Ok(())
}

fn find_section(lines: &[String], start_marker: &str, end_marker: &str) -> Option<(usize, usize)> {
    let start = lines.iter().position(|line| line == start_marker)?;
    let end = lines.iter().position(|line| line == end_marker)?;
    Some((start, end))
}

fn manage_hosts(
    etc_hosts_path: &str,
    v4_hosts_path: &str,
    v6_hosts_path: &str,
    update_v4: bool,
    update_v6: bool,
    remove_v4: bool,
    remove_v6: bool,
) -> io::Result<()> {
    let mut etc_hosts_lines = read_lines(etc_hosts_path)?;
    let v4_hosts_lines = read_lines(v4_hosts_path)?;
    let v6_hosts_lines = read_lines(v6_hosts_path)?;

    let v4_start_marker = "# BEGIN GoogleHosts IPV4";
    let v4_end_marker = "# END GoogleHosts IPV4";
    let v6_start_marker = "# BEGIN GoogleHosts IPV6";
    let v6_end_marker = "# END GoogleHosts IPV6";

    if update_v4 {
        if let Some((start, end)) = find_section(&etc_hosts_lines, v4_start_marker, v4_end_marker) {
            etc_hosts_lines.splice(start + 1..end, v4_hosts_lines.clone());
        } else {
            etc_hosts_lines.push(v4_start_marker.to_string());
            etc_hosts_lines.extend(v4_hosts_lines.clone());
            etc_hosts_lines.push(v4_end_marker.to_string());
        }
    }

    if update_v6 {
        if let Some((start, end)) = find_section(&etc_hosts_lines, v6_start_marker, v6_end_marker) {
            etc_hosts_lines.splice(start + 1..end, v6_hosts_lines.clone());
        } else {
            etc_hosts_lines.push(v6_start_marker.to_string());
            etc_hosts_lines.extend(v6_hosts_lines.clone());
            etc_hosts_lines.push(v6_end_marker.to_string());
        }
    }

    if remove_v4 {
        if let Some((start, end)) = find_section(&etc_hosts_lines, v4_start_marker, v4_end_marker) {
            etc_hosts_lines.drain(start..=end);
        }
    }

    if remove_v6 {
        if let Some((start, end)) = find_section(&etc_hosts_lines, v6_start_marker, v6_end_marker) {
            etc_hosts_lines.drain(start..=end);
        }
    }

    write_lines(etc_hosts_path, &etc_hosts_lines)?;

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} [update-v4 | update-v6 | remove-v4 | remove-v6]", args[0]);
        return Ok(());
    }

    let etc_hosts_path = "/etc/hosts";
    let v4_hosts_path = "data/v4/hosts";
    let v6_hosts_path = "data/v6/hosts";

    let mut update_v4 = false;
    let mut update_v6 = false;
    let mut remove_v4 = false;
    let mut remove_v6 = false;

    for arg in &args[1..] {
        match arg.as_str() {
            "update-v4" => update_v4 = true,
            "update-v6" => update_v6 = true,
            "remove-v4" => remove_v4 = true,
            "remove-v6" => remove_v6 = true,
            _ => {
                eprintln!("Invalid option: {}", arg);
                eprintln!("Usage: {} [update-v4 | update-v6 | remove-v4 | remove-v6]", args[0]);
                return Ok(());
            }
        }
    }

    manage_hosts(
        etc_hosts_path,
        v4_hosts_path,
        v6_hosts_path,
        update_v4,
        update_v6,
        remove_v4,
        remove_v6,
    )?;

    Ok(())
}

