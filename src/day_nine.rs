use actix_web::{post, web, HttpRequest, HttpResponse, Responder};

use log::{error, info};

use leaky_bucket::RateLimiter;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use serde::Deserialize;
use serde_json;

const GALLON_TO_LITER: f32 = 3.7854118;
const PINT_TO_LITRE:   f32 = 0.5682613;

//******************* day 9 **************

#[derive(Deserialize, Debug)]
struct MilkLitersGallonsLitresPints {
    #[serde(default = "default_to_option_f32_if_missing")]
    liters: Option<f32>,
    #[serde(default = "default_to_option_f32_if_missing")]
    gallons: Option<f32>,
    #[serde(default = "default_to_option_f32_if_missing")]
    litres: Option<f32>,
    #[serde(default = "default_to_option_f32_if_missing")]
    pints: Option<f32>,

}

fn default_to_option_f32_if_missing() -> Option<f32> {
    None
}

//actix_web response

#[post("/9/milk")]
pub async fn handle_milk_request(
    bucket: web::Data<Arc<Mutex<RateLimiter>>>,
    body: String,
    req: HttpRequest,
) -> impl Responder {
    // output the post request
    info!("------> Received original body: \n{}", body);
    info!("------> Req is : \n{:#?}", req);

    let content_type = req
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    info!("content_type is: {:?}", content_type);

    let bucket = bucket.lock().await;


    let mut result_output = HttpResponse::Ok().body("initialize".to_string());
    let mut body_out_string: String = "".to_string();

    info!("post request received for Milk Withdrawn! \n");
    let available_tokens = bucket.balance();
    info!("available tokens before milk withdrawn is {}", available_tokens);


    if bucket.try_acquire(1) {
        info!("acquired 1 milk! \n");
        let available_tokens = bucket.balance();
        info!("available tokens after aquire is {}", available_tokens);

        // token acquired, handle http response as Ok
        result_output = HttpResponse::Ok().body("Milk withdrawn\n");
    } else {
        // token is not enough. handle http response as TooManyRequests
        info!("No milk!!!");
        result_output = HttpResponse::TooManyRequests().body("No milk available\n");
    }

    if content_type == "application/json" {
        let json_input_str: Result<MilkLitersGallonsLitresPints, serde_json::Error> =
            serde_json::from_str(&body);
        //info!("json_result is: \n{:#?}", json_result );

        match json_input_str {
            Ok(json_input_str) => {
                info!("json_input_str is: \n{:#?}", json_input_str);

                if name_exist(&json_input_str, "liters") != name_exist(&json_input_str, "gallons"){
                    // either liters or gallons exist in json

                    if name_exist(&json_input_str, "litres") != name_exist(&json_input_str, "pints"){ 
                        // either litres or pints exist in json
                        info!("both liters/gallons  and  litres/pints exist. Bad Request.");
                        result_output = HttpResponse::BadRequest().finish();                  
    
                    } else {
                        info!("liters/gallons exist.   litres/pints not exist. Valid Request.");

                        if name_exist(&json_input_str, "liters"){
                            info!("liters exist, gallons not exist");
                            body_out_string = format!(
                                r#"{{"gallons":{}}}"#,
                                json_input_str.liters.unwrap() / GALLON_TO_LITER
                            );
                            result_output = HttpResponse::Ok().body(body_out_string.clone());
                            info!("{}", body_out_string.clone());
    
                        } else {
                            info!("liters NOT exist, gallons exist");
                            body_out_string = format!(
                                r#"{{"liters":{}}}"#,
                                json_input_str.gallons.unwrap() * GALLON_TO_LITER
                            );
                            result_output = HttpResponse::Ok().body(body_out_string.clone());
                            info!("{}", body_out_string.clone());
    
                        }   
    
                    }


                    

                } else if name_exist(&json_input_str, "litres") != name_exist(&json_input_str, "pints"){ 

                    info!("liters/gallons NOT exist.   litres/pints exist. Valid Request.");

                    if name_exist(&json_input_str, "litres"){
                        info!("litres exist, pints not exist");
                        body_out_string = format!(
                            r#"{{"pints":{}}}"#,
                            json_input_str.litres.unwrap() / PINT_TO_LITRE
                        );
                        result_output = HttpResponse::Ok().body(body_out_string.clone());
                        info!("{}", body_out_string.clone());

                    } else {
                        info!("litres NOT exist, pints exist");
                        body_out_string = format!(
                            r#"{{"litres":{}}}"#,
                            json_input_str.pints.unwrap() * PINT_TO_LITRE
                        );
                        result_output = HttpResponse::Ok().body(body_out_string.clone());
                        info!("{}", body_out_string.clone());

                    }


                } else {

                    info!("both liters/gallons  and  litres/pints exist/not exsit. Bad Request.");
                    result_output = HttpResponse::BadRequest().finish();

                }               
                
            }
            Err(error_info) => {
                error!("Invalid json input. Error_info is: \n{}", error_info);
                body_out_string = "Invalid json input".to_string();
                result_output = HttpResponse::BadRequest().finish();

            }
        }
    }

    result_output
}



fn name_exist(json_results: &MilkLitersGallonsLitresPints, name: &str) -> bool {
    //let mut number: f32 = 0.0;
    let mut name_exist: bool = false;

    match name {
        "liters" => {
            if let Some(liters_number) = json_results.liters {
                info!("liter number is: {}", liters_number);
                //number = liters_number.clone();
                name_exist = true;
            } else {
                info!("No liter input");
                //number = 0.0;
                name_exist = false;
            }
        }

        "gallons" => {
            if let Some(gallons_number) = json_results.gallons {
                info!("gallons number is: {}", gallons_number);
                //number = gallons_number.clone();
                name_exist = true;
            } else {
                info!("No gallon input");
                //number = 0.0;
                name_exist = false;
            }
        }

        "litres" => {
            if let Some(litres_number) = json_results.litres {
                info!("litre number is: {}", litres_number);
                //number = litres_number.clone();
                name_exist = true;
            } else {
                info!("No litre input");
                //number = 0.0;
                name_exist = false;
            }
        }

        "pints" => {
            if let Some(pints_number) = json_results.pints {
                info!("pints number is: {}", pints_number);
                //number = pints_number.clone();
                name_exist = true;
            } else {
                info!("No gallon input");
                //number = 0.0;
                name_exist = false;
            }
        }


        _ => {
            error!("wrong name provided!! valid inputs are liters and/or gallons.");
        }
    }

    name_exist

}




//actix_web response

#[post("/9/refill")]
pub async fn handle_milk_refill( bucket: web::Data<Arc<Mutex<RateLimiter>>> , body: String, req: HttpRequest) -> impl Responder {

    info!("------> Received milk refill post \n");

    // output the post request
    info!("------> Received original body: \n{}", body);
    info!("------> Req is : \n{:#?}", req);

    let content_type = req
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    info!("content_type is: {:?}", content_type);
    
    // create a new bucket
    let  new_bucket = RateLimiter::builder()
        .max(5) // 最大令牌数为 5
        .initial(5) // 初始化时有 5 个令牌
        .interval(Duration::from_secs(1)) // 每 1 秒补充令牌
        .refill(1) // 每次补充 1 个令牌
        .build();

    let mut old_bucket = bucket.lock().await;
    // assign new bucket to old bucket
    *old_bucket = new_bucket;

    info!("------> milk refilled! \n");

    HttpResponse::Ok()

}

