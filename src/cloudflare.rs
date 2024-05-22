use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, HeaderName};
use serde_json::Value;
use std::str::FromStr;

pub(crate) fn cloudflare_update_dns(siteinfo: &Value) -> u8{
    let client_builder: Client = Client::new();
    let mut header_map: HeaderMap<HeaderValue> = HeaderMap::with_capacity(3);
    let headers = siteinfo["header"].as_object().unwrap();
    for (tmp_header_string,header_value) in headers{
        let tmp_header_name: HeaderName = HeaderName::from_str(&tmp_header_string).unwrap();
        header_map.insert(
            tmp_header_name, 
        HeaderValue::from_str(header_value.clone().as_str().unwrap()).expect("Unable to convert to header value")
        );
    }
    let check_dns = client_builder.get(
        siteinfo["url"].as_str().expect("Unable to send get with invalid URL"))
        .headers(header_map.clone())
        .send().expect("Get request failed");
    let get_response_json_string = match check_dns.error_for_status(){
        Ok(r) => r.text().unwrap(),
        Err(e) =>{
            println!("{:?}",e.to_string());
            return 2;
        },
    };

    let mut body_vars_raw = siteinfo["body"].clone();
    let mut body_vars = body_vars_raw.as_object_mut().unwrap().to_owned();

    let get_response_json: Value = serde_json::from_str(&get_response_json_string).expect("Unable to parse json file");
    let dns_ip_resolve = get_response_json["result"]["content"].to_string();
    let ip: String = match body_vars["type"].as_str().unwrap() {
        "A"=>{
            let resp: Value = client_builder.get("https://api4.ipify.org?format=json").send().unwrap().json().expect("Unable to unwrap ipify v4 response");
            resp["ip"].to_string()
        },
        "AAAA"=>{
            let resp: Value = client_builder.get("https://api64.ipify.org?format=json").send().unwrap().json().expect("Unable to unwrap ipify v6 response");
            resp["ip"].to_string()
        }
        _=>{
            println!("Bad record type in json file for {}",body_vars["name"].as_str().unwrap());
            return 2;
        }
    };
    println!("From web: {}\nFrom Cloudflare DNS: {}",ip, dns_ip_resolve);
    if ip != dns_ip_resolve {
        body_vars.insert("content".to_string(), Value::from_str(&ip).unwrap());
        let dns_change_resp: Value = client_builder.patch(
            siteinfo["url"]
            .as_str()
            .expect("Unable to get value with key: 'apiUrl'")
        )
        .headers(header_map)
        .body(serde_json::to_string(&body_vars).unwrap())
        .send().expect("Update DNS failed")
        .json().unwrap();
        let success = dns_change_resp.get("success").expect("Request to server is invalid");
        if success.as_bool().unwrap(){
            return 0;
        }
        else{
            println!("{}",dns_change_resp.to_string())
        }
    }
    else{
        println!("Change not needed");
        return 0;
    }
    return 3;
}