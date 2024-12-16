use actix_web::{ post,  HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde::de::Deserializer;
use indexmap::IndexMap;

use log::{debug, error, info, log_enabled, Level};



//******************* 5 **************

// struct CargoManifest defination

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


// handle the error case when  deserializing toml
#[derive(Serialize, Deserialize, Debug)]
struct Order {
    #[serde(default = "default_to_none_string_if_missing")]
    #[serde(deserialize_with = "deserialize_string_or_none")]
    item: Option<String>,
    #[serde(default = "default_to_none_u32_if_missing")]
    #[serde(deserialize_with = "deserialize_u32_or_none")]
    quantity: Option<u32>,
}

fn default_to_none_string_if_missing() -> Option<String> {
    None
}
fn default_to_none_u32_if_missing() -> Option<u32> {
    None
}

fn deserialize_u32_or_none<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<u32>::deserialize(deserializer) {
        Ok(value) => Ok(value),
        Err(_) => Ok(None), // 类型不匹配时，返回 None
    }
}

fn deserialize_string_or_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<String>::deserialize(deserializer) {
        Ok(value) => Ok(value),
        Err(_) => Ok(None), // 类型不匹配时，返回 None
    }
}





//actix_web response

#[post("/5/manifest")]
pub async fn handle_manifest(body: String) -> impl Responder {
    // Parse the incoming body as a TOML document
    info!("------> Received original Cargo.toml manifest: \n{}", body);
    match toml::from_str::<CargoManifest>(&body) {
        Ok(manifest) => {
            info!("Match Cargo.toml manifest: \n{:?}", manifest);



            let body_out_string = data_processing(manifest); 

            info!(
                "decoded info to be sent to  body_output to string: \n{:?}",
                body_out_string
            );



            if body_out_string.is_empty() {
                info!("No valid orders");
                HttpResponse::NoContent().finish()
            } else {
                HttpResponse::Ok().body(body_out_string) // Return parsed manifest as
            }
        }
        Err(err) => {
            error!("erro with toml input:  \n {}", err);
            //eprintln!("Failed to parse TOML: {}", err);
            //HttpResponse::NoContent().body(format!("Invalid TOML: {}", err))

            HttpResponse::NoContent().finish()
        }
    }
}






//processing the CargoManifest Stuct into String
fn data_processing(manifest:CargoManifest) -> String {

    
    //use indexmap to store the CargoManifest data, as Hashmap data order is arbitrary 
    //and BTreemap data is ordered from small to big 
    let mut indexmap_init: IndexMap<Option<String>, Option<u32>> = IndexMap::new();

    //step 0#
    // put raw Order information into IndexMap
    for one_order in &manifest.package.metadata.orders {
        info!(
            "here {:?}:{:?}\n",
            one_order.item.clone(),
            one_order.quantity.clone()
        );
        indexmap_init.insert(one_order.item.clone(), one_order.quantity.clone());

    }


    info!(
        "decoded info to be sent to  hash_output before processing: \n{:?}",
        indexmap_init
    );


    // Step 1#
    //  filter out "None" value datas
    indexmap_init.retain(|_, value| value.is_some());


    // Step 2#
    //decode Option<T> --> T
    //BTreeMap<Option<String>, Option<u32>> -->  BTreeMap<String, u32>

    let indexmap_string_u32: IndexMap<String, u32> = indexmap_init
        .into_iter()
        .filter_map(|(key2, value2)| {
            if let (Some(k), Some(v)) = (key2, value2) {
                Some((k, v))
            } else {
                None
            }
        })
        .collect();


    info!(
        "decoded info to be sent to  indexmap_string_u32: \n{:?}",
        indexmap_string_u32
    );


    //Step 3#
    // indexmap<String, u32>  --> Vec<String>

    let mut body_out_vec_string: Vec<String> = Vec::new();
    for (key3, value3) in indexmap_string_u32 {
        body_out_vec_string.push(format!("{}: {}", key3, value3));
    }

    // Step 4#
    // Vec<String>  --> String
    body_out_vec_string.join("\n")



}