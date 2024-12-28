use actix_web::{cookie::Cookie, get, post, HttpRequest, HttpResponse, Responder};
use jsonwebtoken::{
    decode, encode, errors::ErrorKind, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use log::{error, info};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

use std::fs::File;
use std::io::{self, Read};
use std::str;

use std::collections::HashSet;
use std::env;
use std::fs;



/// 定义 JWT 的负载结构
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // 用户ID或身份标识
    exp: usize,  // 到期时间 (UNIX时间戳)
}

/// 定义 JWT 的负载结构
#[derive(Debug, Serialize, Deserialize)]
struct ClaimsJsonValue {
    contents: serde_json::Value,
    exp: usize,
}

/// 秘钥，用于签名和验证
const SECRET_KEY: &[u8] = b"supersecretkey";

//actix_web response

#[post("/16/wrap")]
pub async fn wrap_gift(body: String, req: HttpRequest) -> impl Responder {
    // output the get request
    info!(
        "============================> Received original body in post /16/wrap: \n{}",
        body
    );
    info!("------> Req is : \n{:#?}", req);

    let content_type = req
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    info!("content_type is: {:?}", content_type);

    let json_input_str: Value = serde_json::from_str(&body).unwrap();
    info!("json_input_str is: {:?}", json_input_str);
    //let mut json_output_str = "initial".to_string();

    let json_output_str = serde_json::to_string(&json_input_str).unwrap();

    // now time
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as usize;

    // JWT valid time
    let expiration_time = now + 3600;

    // crate Claims JWT payload

    let claims = Claims {
        sub: json_output_str.clone(),
        exp: expiration_time,
    };

    // generate JWT
    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(SECRET_KEY),
    )
    .expect("Error when generating JWT");

    // creat cookie
    let cookie = Cookie::build("gift", token)
        .path("/")
        .http_only(true)
        .secure(false)
        .finish();

    return HttpResponse::Ok().cookie(cookie).body(json_output_str);
}


/// unwrap the tokens
#[get("/16/unwrap")]
pub async fn unwrap_gift(body: String, req: HttpRequest) -> impl Responder {
    // output the get request
    info!(
        "=========================================> Received original body in get /16/unwrap: \n{}",
        body
    );
    info!("------> Req is : \n{:#?}", req);

    let content_type = req
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    info!("content_type is: {:?}", content_type);

    let mut http_output = HttpResponse::Ok().body("hello");

    if let Some(cookie) = req.cookie("gift") {
        let token = cookie.value();
        info!("token is: {:?}", token);

        if token == "" {
            return HttpResponse::BadRequest().body("missing token!");
        }

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(SECRET_KEY),
            &Validation::new(Algorithm::HS256),
        );

        match token_data {
            Ok(data) => {
                let claims = data.claims;
                info!("claim.sub is \n{:?}", claims.sub);
                http_output = HttpResponse::Ok().body(claims.sub);
            }
            Err(error_info) => {
                error!("missing valid cookie!!");
                http_output = HttpResponse::BadRequest().body("Missing gift Cookie");
            }
        }
    } else {
        info!("token is: empty!!!");
        return HttpResponse::BadRequest().body("missing token!");
    }

    return http_output;
}


/// decode the tokens
#[post("/16/decode")]
pub async fn decode_jwt(body: String, req: HttpRequest) -> impl Responder {
    // output the get request
    info!(
        "===============================> Received original body in post /16/decode: \n{}",
        body
    );
    info!("------> Req is : \n{:#?}", req);

    let content_type = req
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    info!("content_type is: {:?}", content_type);

    //let mut file = File::open("./src/day16_santa_public_key.pem")?;

    let current_path = match env::current_dir() {
        Ok(path) => path,
        Err(e) => {
            error!("无法获取当前路径: {}", e);
            return HttpResponse::InternalServerError()
                .body(format!("Failed to read public key file: {}", e));
        }
    };

    println!("当前路径是: {}", current_path.display());
    

    match fs::read_dir(&current_path) {
        Ok(entries) => {
            println!("当前路径下的文件和目录有:");
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        if path.is_dir() {
                            println!("目录: {}", path.display());
                        } else {
                            println!("文件: {}", path.display());
                        }
                    }
                    Err(e) => println!("读取文件或目录失败: {}", e),
                }
            }
        }
        Err(e) => println!("无法读取当前路径: {}", e),
    }


/*     let mut file = match File::open("../assets/day16_santa_public_key.pem") {
        Ok(file) => file,
        Err(err) => {
            // 文件读取失败，返回错误响应
            info!("Error loading pem file!!!!!");
            return HttpResponse::InternalServerError()
                .body(format!("Failed to read public key file: {}", err));
        }
    };

    let mut santa_key_pem = Vec::new();
    file.read_to_end(&mut santa_key_pem)
        .expect("Unable to read public key file"); */

    //let public_key_pem: = fs::read("./src/day16_santa_public_key.pem").expect("Failed to read day16 Sata public key file");

    let santa_key_pem_string = b"-----BEGIN PUBLIC KEY-----\n\
                           MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAs5BlLjDtKuEY2NV3+xhH\n\
                           WWlKrZDWkIOV+HoLURIBEpAHa11xU+wL9sySR17j4bL9MJawlCJAGArW8vnDiAv8\n\
                           366PfOhCqZsD9N2iG28y7vf5q1PhoXl/Vfuelykw0k+r4054h0uCg9Olal0Nm/V8\n\
                           vsdPEC3wjNLBi86oYESkW43/7lbBWPBti1POCVJDuBEASZFhIR2+mfz6AFWQwmqO\n\
                           zzhP1Yli/7EtNMELWezQJXnVLQ3JvjT2btWWwKYT468YX/NtQgMC7SLvIRBuWb/Z\n\
                           ayfoi/9rGndqW0YPE1xwJEQA415w5HbfTneyAIxDy7TC8/+dFaKRcoPiEQA1T5bk\n\
                           OQIDAQAB\n-----END PUBLIC KEY-----";
    
    let santa_key_pem = santa_key_pem_string;



    // 示例的 JWT
    let token = &body;

    // 使用 RSA 公钥创建 DecodingKey
    info!("ddecoding_key==1");
    let decoding_key = DecodingKey::from_rsa_pem(santa_key_pem).expect("Invalid RSA key");
    //info!("ddecoding_key:\n=={:?}==", decoding_key);
    info!("ddecoding_key==2");

    //验证规则（指定算法为 RS256）

    let mut validation = Validation::new(Algorithm::RS256);
    validation.required_spec_claims = HashSet::new();
    validation.algorithms = vec![Algorithm::RS256, Algorithm::RS512];

    // let validation = Validation {
    //     required_spec_claims: HashSet::new(),
    //     leeway: 60,
    //     validate_exp: true,
    //     validate_nbf: false,
    //     algorithms: vec![Algorithm::RS256, Algorithm::RS512],
    //     ..Default::default()
    // };

    // let validation = Validation {
    //     algorithms: vec![Algorithm::RS256, Algorithm::RS512],
    //     ..Default::default()
    // };

    let mut output_str = "initial".to_string();
    let mut http_output = HttpResponse::Ok().body(output_str);

    // Value
    info!("decoding token:\n=={}==", token);
    match decode::<Value>(token, &decoding_key, &validation) {
        
        Ok(token_data) => {
            info!("Decoded token as serde_json::Value : {:?}", token_data.claims);
            output_str = serde_json::to_string(&token_data.claims).unwrap();
            http_output = HttpResponse::Ok().body(output_str);
        }
        Err(e) => match e.kind() {
            ErrorKind::Json(_) => {
                info!("error Json");
                http_output = HttpResponse::BadRequest().body("");
            }
            ErrorKind::ExpiredSignature => {
                info!("ExpiredSignature");
                http_output = HttpResponse::BadRequest().body("");
            }
            ErrorKind::InvalidToken => {
                info!("Invalid token");
                http_output = HttpResponse::BadRequest().body("bad request");
            }
            ErrorKind::InvalidSignature => {
                info!("InvalidSignature!!!");
                http_output = HttpResponse::Unauthorized().body("invalid signature");
            }
            ErrorKind::InvalidAlgorithm => {
                info!("InvalidAlgorithm!!!");
                http_output = HttpResponse::BadRequest().body("InvalidAlgorithm");
            }
            _ => {
                error!("Other error: {:?}", e);

                http_output = HttpResponse::BadRequest().body("bad request");
            }
        },
    }

    return http_output
}
