use log::info;
use sqlx::types::time::OffsetDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{NaiveDateTime, DateTime, Utc};


use actix_web::{
    error, get,  HttpResponse, Responder,
    post, delete, put,
    web::{self, Json, ServiceConfig},
    Result,
};
use sqlx::{FromRow, PgPool};




#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct Quote {
    pub id: Uuid,
    pub author: String,
    pub quote: String,
    //pub created_at: OffsetDateTime,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub version: i32,
}

#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct UpdateQuote {
    pub author: String,
    pub quote: String,
}

// #[derive(Clone)]
// pub struct DBClinet {
//     pool: Pool<Postgres>,
// }






#[post("/19/reset")]
// Reset the database: Clear all quotes
pub async fn reset(pool_data: web::Data<PgPool>) -> Result<HttpResponse, actix_web::Error> {
    info!("reset post received");
    // 从连接池获取连接
    let mut conn = pool_data.acquire().await.map_err(|e| error::ErrorInternalServerError(e.to_string()))?;

    // 执行 TRUNCATE 操作
    sqlx::query("TRUNCATE TABLE quotes")
        .execute(&mut *conn)
        .await
        .map_err(|e| error::ErrorInternalServerError(e.to_string()))?;

    // 返回成功响应
    Ok(HttpResponse::Ok().finish())
}



#[get("/19/cite/{id}")]
// Get a quote by ID
pub async fn get_quote(id: web::Path<Uuid>, pool_data: web::Data<PgPool>) -> Result<HttpResponse, actix_web::Error> {
    info!("get  cite id  received");
    // 获取数据库连接
    let mut conn = pool_data.acquire().await.unwrap();

    // 查询数据库
    let quote = sqlx::query_as::<_, Quote>(
        "SELECT id, author, quote, created_at, version FROM quotes WHERE id = $1"
    )
    .bind(*id) // 使用 id 作为参数绑定
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| error::ErrorNotFound(e.to_string()))?;

    // 返回结果
    Ok(HttpResponse::Ok().json(quote))
}




#[delete("/19/remove/{id}")]
// Delete a quote by ID
pub async fn delete_quote(pool_data: web::Data<PgPool>, id: web::Path<Uuid>) -> impl Responder {
    info!("delete  remove id  received");
    let mut conn = match pool_data.acquire().await {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().finish(),  // 如果数据库连接失败，返回 500 错误
    };

    // 查询要删除的记录
    let existing_quote = sqlx::query_as::<_, Quote>(
        "SELECT id, author, quote, created_at, version FROM quotes WHERE id = $1"
    )
    .bind(*id)
    .fetch_optional(&mut *conn)
    .await;


    match existing_quote {
        Ok(Some(quote)) => {
            // 如果记录存在，执行删除操作
            let result = sqlx::query("DELETE FROM quotes WHERE id = $1")
                .bind(*id)
                .execute(&mut *conn)
                .await;

            match result {
                Ok(deleted) => {
                    if deleted.rows_affected() == 0 {
                        // 如果没有删除任何记录（理论上不会发生），返回 404
                        info!("/19/remove: record not found!");
                        HttpResponse::NotFound().finish()
                    } else {
                        // 删除成功，返回被删除的记录
                        info!("/19/remove: record deleted successfully");
                        HttpResponse::Ok().json(quote)
                    }
                }
                Err(_) => {
                    // SQL 执行失败时返回 500
                    info!("/19/remove: delete failed, internal server error");
                    HttpResponse::InternalServerError().finish()
                }
            }
        }
        Ok(None) => {
            // 如果记录不存在，返回 404
            info!("/19/remove: record not found!");
            HttpResponse::NotFound().finish()
        }
        Err(_) => {
            // 查询失败时返回 500
            info!("/19/remove: query failed, internal server error");
            HttpResponse::InternalServerError().finish()
        }
    }
}





#[put("/19/undo/{id}")]
// Update a quote by ID
pub async fn update_quote(
    pool_data: web::Data<PgPool>,
    id: web::Path<String>,
    new_quote: web::Json<UpdateQuote>,
) -> Result<HttpResponse, actix_web::Error> {

    info!("put  undo id  received");
    // 尝试解析 UUID，如果无效则返回 BadRequest
    let parsed_id = Uuid::parse_str(&id).map_err(|_| {
        error::ErrorBadRequest("Invalid UUID format")  // 如果解析失败，返回 BadRequest
    })?;

    let mut conn = match pool_data.acquire().await {
        Ok(conn) => conn,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),  // 数据库连接失败，返回 500
    };

    // 执行 SQL 更新操作
    let updated = sqlx::query_as::<_, Quote>(
        "UPDATE quotes SET author = $1, quote = $2, version = version + 1 WHERE id = $3 RETURNING id, author, quote, created_at, version"
    )
    .bind(&new_quote.author)   // 使用 .bind() 来绑定参数
    .bind(&new_quote.quote)    // 使用 .bind() 来绑定参数
    .bind(parsed_id)                 // 绑定 ID
    .fetch_optional(&mut *conn)  // 返回一个可选的结果
    .await;

    match updated {
        Ok(Some(quote)) => {
            // 更新成功，返回更新后的 quote
            Ok(HttpResponse::Ok().json(quote)) // 返回查询到的 quote
        },
        Ok(None) => {
            // 没有找到该 ID 的记录，返回 404
            Ok(HttpResponse::NotFound().finish())
        },
        Err(_) => {
            // 数据库操作失败，返回 500
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}




#[post("/19/draft")]
// Add a new quote
pub async fn add_quote(pool_data: web::Data<PgPool>, new_quote: web::Json<UpdateQuote>) -> impl Responder {
    info!("post  draft   received");
    let id = Uuid::new_v4();
    //let created_at = NaiveDateTime::from_timestamp(chrono::Utc::now().timestamp(), 0); // 使用 NaiveDateTime 来表示时间
    //let created_at: DateTime<Utc> = Utc::now();
    let created_at = chrono::Utc::now();

    // 获取数据库连接
    let mut conn = pool_data.acquire().await.unwrap();

    // 执行插入查询，返回新插入的行
    let inserted = sqlx::query_as::<_, Quote>(
        "INSERT INTO quotes (id, author, quote, created_at) 
         VALUES ($1, $2, $3, $4) 
         RETURNING id, author, quote, created_at, version"
    )
    .bind(id)         // 绑定 ID
    .bind(&new_quote.author)   // 绑定 author
    .bind(&new_quote.quote)    // 绑定 quote
    .bind(created_at) // 绑定 created_at
    .fetch_one(&mut *conn)     // 执行并获取一行结果
    .await
    .unwrap(); // 异常处理：可以根据需要改成更优雅的错误处理

    // 返回插入后的 quote 数据
    HttpResponse::Created().json(Quote {
        id: inserted.id,
        author: inserted.author,
        quote: inserted.quote,
        created_at: inserted.created_at,
        version: inserted.version,
    })
}



