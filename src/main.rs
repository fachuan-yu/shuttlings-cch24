use actix_web::{
    get, http::{header::DATE, StatusCode}, post, web::{self, ServiceConfig}, App, HttpResponse, HttpResponseBuilder,
    HttpServer, Responder,
};
use serde::{de::MapAccess, Deserialize, Serialize};
use shuttle_actix_web::ShuttleActixWeb;

use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

//-1
#[get("/")]
async fn hello_bird() -> &'static str {
    "Hello, bird!"
}

#[get("/-1/seek")]
async fn seek() -> HttpResponse {
    HttpResponseBuilder::new(StatusCode::from_u16(302).unwrap())
        .insert_header(("Location", "https://www.youtube.com/watch?v=9Gc4QTqslN4"))
        .finish()
}

//2
// curl "http://localhost:8000/2/dest?from=10.0.0.0&key=1.2.3.255"
// 11.2.3.255
//curl "http://localhost:8000/2/dest?from=128.128.33.0&key=255.0.255.33"
//127.128.32.33
//
#[derive(Deserialize, Debug)]
struct DestKey {
    from: Ipv4Addr,
    key: Ipv4Addr,
}

#[get("/2/dest")]
async fn dest(addr: web::Query<DestKey>) -> String {
    let from_bytes = addr.from.octets();
    let key_bytes = addr.key.octets();

    let sum = [
        from_bytes[0].wrapping_add(key_bytes[0]),
        from_bytes[1].wrapping_add(key_bytes[1]),
        from_bytes[2].wrapping_add(key_bytes[2]),
        from_bytes[3].wrapping_add(key_bytes[3]),
    ];

    format!("{}", Ipv4Addr::from(sum).to_string())
}

// curl "http://localhost:8000/2/key?from=10.0.0.0&to=11.2.3.255"
// 1.2.3.255
// curl "http://localhost:8000/2/key?from=128.128.33.0&to=127.128.32.33"
// 255.0.255.33

#[derive(Deserialize, Debug)]
struct FromTo {
    from: Ipv4Addr,
    to: Ipv4Addr,
}

#[get("/2/key")]
async fn key(addr: web::Query<FromTo>) -> String {
    let from_bytes = addr.from.octets();
    let to_bytes = addr.to.octets();

    let sub = [
        to_bytes[0].wrapping_sub(from_bytes[0]),
        to_bytes[1].wrapping_sub(from_bytes[1]),
        to_bytes[2].wrapping_sub(from_bytes[2]),
        to_bytes[3].wrapping_sub(from_bytes[3]),
    ];

    format!("{}", Ipv4Addr::from(sub).to_string())
}

// curl "http://localhost:8000/2/v6/dest?from=fe80::1&key=5:6:7::3333"
//fe85:6:7::3332

#[derive(Deserialize, Debug)]
struct DestKeyV6 {
    from: Ipv6Addr,
    key: Ipv6Addr,
}

#[get("/2/v6/dest")]
async fn V6Dest(addr: web::Query<DestKeyV6>) -> String {
    let from_bytes = addr.0.from.octets();
    let key_bytes = addr.0.key.octets();

    let sum_result: [u8; 16] = from_bytes
        .iter()
        .zip(key_bytes.iter())
        .map(|(a, b)| a ^ b)
        .collect::<Vec<u8>>()
        .try_into()
        .expect("XOR result should always be 16 bytes");

    format!("{}", Ipv6Addr::from(sum_result).to_string())
}

//curl "http://localhost:8000/2/v6/key?from=aaaa::aaaa&to=5555:ffff:c:0:0:c:1234:5555"
//ffff:ffff:c::c:1234:ffff
#[derive(Deserialize, Debug)]
struct FromToV6 {
    from: Ipv6Addr,
    to: Ipv6Addr,
}

#[get("/2/v6/key")]
async fn V6Key(addr: web::Query<FromToV6>) -> String {
    let from_bytes = addr.0.from.octets();
    let to_bytes = addr.0.to.octets();

    let sub_result: [u8; 16] = from_bytes
        .iter()
        .zip(to_bytes.iter())
        .map(|(a, b)| a ^ b)
        .collect::<Vec<u8>>()
        .try_into()
        .expect("XOR result should always be 16 bytes");

    format!("{}", Ipv6Addr::from(sub_result).to_string())
}

//******************* 5 **************

#[derive(Serialize, Deserialize, Debug)]
struct CargoManifest {
    package: Package,
}
#[derive(Serialize, Deserialize, Debug)]
struct Package {
    name: String,
    //version: String,
    authors: Option<Vec<String>>,
    keywords: Option<Vec<String>>,
    metadata: Metadata, //
}
#[derive(Serialize, Deserialize, Debug)]
struct Metadata {
    orders: Vec<Order>,
}
#[derive(Serialize, Deserialize, Debug)]
struct Order {
    item: String,
    quantity: u32,
}

#[post("/5/manifest")]
async fn handle_manifest(body: String) -> impl Responder {
    // Parse the incoming body as a TOML document
    match toml::from_str::<CargoManifest>(&body) {
        Ok(manifest) => {
            println!("Received valid Cargo.toml manifest: {:?}", manifest);
            let mut hash_output = HashMap::new();

            for i in &manifest.package.metadata.orders{
                hash_output.insert(i.item.clone(), i.quantity);

            }
            // hash_output.insert(
            //     String::from(&manifest.package.metadata.orders.get(0).unwrap().item),
            //     &manifest.package.metadata.orders.get(0).unwrap().quantity,
            // );
            // hash_output.insert(
            //     String::from(&manifest.package.metadata.orders.get(1).unwrap().item),
            //     &manifest.package.metadata.orders.get(1).unwrap().quantity,
            // );

            println!("manifest: {:?}", hash_output);
            let body = hash_output
            .iter()
            .map(|(KeyName, value)| format!("{}: {}", KeyName, value))
            .collect::<Vec<String>>()
            .join("\n");

            HttpResponse::Ok()
            .content_type("text/plain")
            .body(body) // Return parsed manifest as 
        }
        Err(err) => {
            //eprintln!("Failed to parse TOML: {}", err);
            //HttpResponse::NoContent().body(format!("Invalid TOML: {}", err))
            HttpResponse::NoContent().finish()
        }
    }
}

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(hello_bird)
            .service(seek)
            .service(dest)
            .service(key)
            .service(V6Dest)
            .service(V6Key)
            .service(handle_manifest);
    };

    Ok(config.into())
}
