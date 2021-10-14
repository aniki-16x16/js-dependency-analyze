pub mod utils;

use std::collections::HashMap;

use clap::{App, Arg};
use regex::Regex;
use utils::FileRecoder;

fn main() {
    let matches = App::new("JS Dependency Analyzer")
        .version("0.1.0")
        .arg(
            Arg::with_name("path")
                .help("root file")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("alias")
                .help("alias path")
                .short("a")
                .long("alias")
                .takes_value(true),
        )
        .get_matches();

    let re =
        Regex::new(r#"(?:import.*['"](.*)['"]|require\(['"](.*)['"]\)|\s+from\s+['"](.*)['"])"#)
            .unwrap();
    let root = FileRecoder::new(matches.value_of("path").unwrap());
    let mut paths = vec![root.clone()];
    let mut alias = HashMap::new();
    for item in matches
        .value_of("alias")
        .map_or(vec![], |x| x.split(',').collect())
    {
        let pair = item.split('=').collect::<Vec<_>>();
        alias.insert(pair[0], pair[1]);
    }
    let mut ref_count = HashMap::new();

    while paths.len() > 0 {
        let mut cur_path = paths.pop().unwrap();
        println!("read {}\nrest {} modules", cur_path.to_string(), paths.len());
        let lines = cur_path.read_line();
        for line in &lines {
            let caps = re.captures(line);
            if caps.is_some() {
                let caps = caps.unwrap();
                let mut name = if caps.get(1).is_some() {
                    caps.get(1).unwrap()
                } else if caps.get(2).is_some() {
                    caps.get(2).unwrap()
                } else {
                    caps.get(3).unwrap()
                }
                .as_str()
                .to_string();

                let mut is_relative_path = None;
                if name.starts_with("./") || name.starts_with("../") {
                    is_relative_path.replace(true);
                } else {
                    for (k, v) in &alias {
                        if name.starts_with(k) {
                            is_relative_path.replace(false);
                            name = name.replacen(k, v, 1);
                            break;
                        }
                    }
                }
                if is_relative_path.is_some() {
                    let mut next_path = match is_relative_path.unwrap() {
                        true => cur_path.join(&name),
                        false => root.join(&name),
                    };
                    if next_path.complete_path() {
                        name = next_path.to_string();
                        if ref_count.get(&name).is_none() {
                            paths.push(next_path);
                        }
                    }
                }
                *ref_count.entry(name.clone()).or_insert(0) += 1;
            }
        }
    }
    println!("{:#?}", ref_count);
}
