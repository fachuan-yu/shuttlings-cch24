use actix_web::{get, web, };

use serde::Deserialize;

use std::net::{Ipv4Addr, Ipv6Addr};



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
pub async fn dest(addr: web::Query<DestKey>) -> String {
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
pub async fn key(addr: web::Query<FromTo>) -> String {
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
pub async fn v6_dest(addr: web::Query<DestKeyV6>) -> String {
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
pub async fn v6_key(addr: web::Query<FromToV6>) -> String {
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


