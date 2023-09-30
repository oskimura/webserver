mod parser;

use actix_web::{test, get, App, HttpServer, Responder, HttpResponse, post, web};
use serde::{Deserialize, Serialize};
use crate::parser::{convert_select_statement, parse_select, traverse_select_statement};
use actix_rt::System;
use serde_json::json;
use actix_web::dev::Service;
use actix_web::middleware::Logger;

#[derive(Debug,Clone,PartialEq, Deserialize,Serialize)]
struct SqlParameter {
    sql: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Todo {
    id: i32,
    content: String,
    checked: bool,
}
#[get("/api/test")]
async fn test_index() -> impl Responder {
    HttpResponse::Ok().json(Todo {
        id: 1,
        content: "test".to_string(),
        checked: false,
    })
}

#[post("/api/parse")]
async fn parse(req_body: web::Json<SqlParameter>) -> impl Responder {
    let sql = String::from(&req_body.sql);
    match parse_select(&String::from(sql)) {
        Ok((_, ast)) =>
            HttpResponse::Ok().body(traverse_select_statement(convert_select_statement(&ast))),
        Err(e) => HttpResponse::Ok().body(e.to_string())
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "axtix_web=info");

    HttpServer::new(|| {
        App::new()
            .service(test_index)
            .service(parse)
            .wrap(Logger::default())
    })
        .bind(("web",8080))?
        .run()
        .await
}
