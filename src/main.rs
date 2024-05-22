use serde_json::Value;
use std::fs::File;
use std::io::prelude::*;
use std::process::exit;
use glob::glob;

mod cloudflare;
use cloudflare::cloudflare_update_dns;

fn main() {
    for filepath in glob("*.json").expect("Failed to read glob pattern") {
        let path = match filepath {
            Ok(path) => path,
            Err(_) => exit(1),
        };
        let display = path.display();
        let mut file = match File::open(&path) {
            Err(why) => panic!("couldn't open {}: {}", display, why),
            Ok(file) => file,
        };
        let mut s = String::new();
        match file.read_to_string(&mut s) {
            Err(why) => panic!("couldn't read {}: {}", display, why),
            Ok(_) => (),
        }
        let v: Value = serde_json::from_str(&s).expect("Unable to parse json file");
        v["sites"].as_array().unwrap().iter().for_each(
            |siteinfo: &Value|{
                let ret:u8 = match siteinfo["provider"].as_str().unwrap(){
                    "cloudflare"=>{
                        cloudflare_update_dns(siteinfo)
                    },
                    _=>{
                        println!("Unrecognized provider \"{}\" for site {}",siteinfo["provider"].as_str().unwrap(),siteinfo["body"]["name"].as_str().unwrap());
                        exit(3);
                    }
                };
                if ret != 0{
                    exit(ret.into());
                }
            }
        );
    }
    exit(0);
}
