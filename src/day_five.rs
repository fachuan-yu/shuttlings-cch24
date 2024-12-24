use std::{str::FromStr};

use actix_web::{
    post, HttpResponse, Responder, HttpRequest,
};

use indexmap::IndexMap;

use cargo_manifest::Manifest;
use toml::Value;

use serde_json;
use serde_yaml;

use serde::de::Deserializer;
use serde::{Deserialize, Serialize};

use log::{ error, info, warn};

//******************* 5 **************


//actix_web response

#[post("/5/manifest")]
pub async fn handle_manifest(body: String, req: HttpRequest) -> impl Responder {
    // Parse the incoming body as a TOML document
    info!("------> Received original manifest: \n{}", body);
    info!("------> Req is : \n{:#?}", req);



    // initialization
    //let mut status_code_output = MyHttpResponseStatus::OkStatus(actix_web::http::StatusCode::OK);
    let mut body_out_string: String = "".to_string();

    let mut result_output = HttpResponse::Ok().body("initialization".to_string());



    // parser content_type: 
    // toml
    // html
    // ymal
    // json
    let content_type = req
    .headers()
    .get("content-type")
    .and_then(|v| v.to_str().ok())
    .unwrap_or("");


    match content_type {
        "application/toml" => {
            info!("Matched application/toml");
            parser_toml(&body)
        },

        "text/html" => {
            info!("Matched text/html, response is UnsupportedMediaType!");

            body_out_string = "".to_string();
            result_output = HttpResponse::UnsupportedMediaType().body(body_out_string);
            result_output
        },

        "application/yaml" => {
            info!("Matched application/ymal");

            let manifest_yaml_result: Result<Manifest, serde_yaml::Error> = serde_yaml::from_str::<Manifest>(&body);
            match manifest_yaml_result{
                Ok(cargo_manifest_yaml) =>{
                    info!("cargo_manifest is: \n{:#?}", cargo_manifest_yaml);

                    let serialized_cargo_manifest = toml::to_string(&cargo_manifest_yaml).unwrap();

                    result_output = parser_toml(&serialized_cargo_manifest);

                },
                Err(error_info) => {
                    error!("error_info is: \n{}", error_info);
                    body_out_string = "Invalid manifest".to_string();
                    result_output = HttpResponse::BadRequest().body(body_out_string);

                }


            }
            // output
            result_output

        },

        "application/json" => {
            info!("Matched application/json");

            let manifest_yaml_result: Result<Manifest, serde_json::Error> = serde_json::from_str::<Manifest>(&body);
            match manifest_yaml_result{
                Ok(cargo_manifest_yaml) =>{
                    info!("cargo_manifest is: \n{:#?}", cargo_manifest_yaml);

                    let serialized_cargo_manifest = toml::to_string(&cargo_manifest_yaml).unwrap();

                    result_output = parser_toml(&serialized_cargo_manifest);

                },
                Err(error_info) => {
                    error!("error_info is: \n{}", error_info);
                    body_out_string = "Invalid manifest".to_string();
                    result_output = HttpResponse::BadRequest().body(body_out_string);

                }


            }
            // output
            result_output

        },

        _ => {
            error!("Matched something else!");
            parser_toml(&body)
        },

    } // match content_type {



}




fn parser_toml(body: &str) -> HttpResponse{


    // initialization
    //let mut status_code_output = MyHttpResponseStatus::OkStatus(actix_web::http::StatusCode::OK);
    let mut body_out_string: String = "".to_string();

    let mut result_output = HttpResponse::Ok().body("initialization".to_string());

    let mut body_out_vec_string: Vec<String> = Vec::new();
        
    
    // let manifest = Manifest::from_str(&body);
    match Manifest::from_str(&body) {
        Ok(manifest) => {
            info!("original manifest is:  \n{:#?}", manifest);

            //let manifest_value: toml::Value = toml::de::from_str(toml_str).unwrap();

            if let Some(package_value) = manifest.package {
                info!("manifest package is \n{:?}", package_value);

                // judge if the "kewords" is "Christmas 2024"
                //
                if let Some(keywords_value_local) = package_value.keywords {
                    info!("keywords_value_out is: {:?}\n", keywords_value_local);

                    if let Some(keywords_vec) = keywords_value_local.as_local() {
                        info!("keywords_vec is: {:?}\n", keywords_vec);

                        for keyword in keywords_vec {
                            info!("keyword is: {:?}\n", keyword);
                            if keyword == "Christmas 2024" {
                                info!("keyword is correct: {}\n", keyword);

                                // ------------------
                                // judge if there is any "metadata table"?
                                if let Some(metadata_table) = package_value.metadata.clone() {
                                    info!("metadata table is \n{:?}", metadata_table);

                                    // judge if there is any "orders" in metadata table?
                                    if let Some(Value::Array(orders)) = metadata_table.get("orders")
                                    {
                                        for order in orders {
                                            if let Value::Table(order_table) = order {
                                                let item = order_table
                                                    .get("item")
                                                    .and_then(|v| v.as_str())
                                                    .unwrap_or("unknown");
                                                let quantity = order_table
                                                    .get("quantity")
                                                    .and_then(|v| v.as_integer())
                                                    .unwrap_or(0);

                                                println!(
                                                    "Item: {}, Quantity: {}\n",
                                                    item, quantity
                                                );


                                                if quantity == 0 {
                                                    info!("{}'s quantity is 0!!!!!", item);
                                                } else {
                                                    body_out_vec_string
                                                        .push(format!("{}: {}", item, quantity));
                                                }
                                            } else {
                                                // if let Value::Table(order_table)= order {
                                                info!("no order in orders!");
                                            }
                                        } //for order in orders {

                                        let body_out_string: String =
                                            body_out_vec_string.join("\n");
                                        info!("body out string is:\n{}", body_out_string);

                                        //let body_out_string: String = body_out_vec_string.iter().map(|s| *s).collect();

                                        if body_out_string == "" {
                                            result_output =
                                                HttpResponse::NoContent().body(body_out_string);
                                        } else {
                                            result_output =
                                                HttpResponse::Ok().body(body_out_string);
                                        }
                                    } else {
                                        //if let Some(Value::Array(orders) ) = metadata_table.get("orders") {
                                        info!("no orders in metadata!");
                                        body_out_string = "".to_string();
                                        result_output =
                                            HttpResponse::NoContent().body(body_out_string);
                                    }
                                } else {
                                    // if let Some(metadata_table) = package.metadata{
                                    info!("metadata is None!");
                                    body_out_string = "".to_string();
                                    result_output = HttpResponse::NoContent().body(body_out_string);
                                }
                            } else {
                                info!("keyword is not correct: {}\n", keyword);
                                body_out_string = "Magic keyword not provided".to_string();
                                result_output = HttpResponse::BadRequest().body(body_out_string);
                            }
                        }
                    }
                } else {  // if let Some(keywords_value_local) = package_value.keywords {
                    warn!("keywords field is missing! Magic keyword not provided!");
                    body_out_string = "Magic keyword not provided".to_string();
                    result_output = HttpResponse::BadRequest().body(body_out_string);


                }
            }
        }
        Err(error_info) => {
            error!("error loading manifest in toml: {}", error_info);

            body_out_string = "Invalid manifest".to_string();
            result_output = HttpResponse::BadRequest().body(body_out_string);


        }
    }

    // parser_toml return
    result_output


}