mod parser;

use actix_web::{get, App, HttpServer, Responder, HttpResponse, post, web};
use serde::{Deserialize, Serialize};
use crate::parser::{convert_select_statement, parse_select};
#[derive(Deserialize)]
struct SqlParameter {
    sql: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Todo {
    id: i32,
    content: String,
    checked: bool,
}
#[get("/api/todo")]
async fn todo_index() -> impl Responder {
    HttpResponse::Ok().json(Todo {
        id: 1,
        content: "やることはapi".to_string(),
        checked: false,
    })
}

#[post("/api/parse")]
async fn parse(req_body: web::Json<SqlParameter>) -> impl Responder {
    let sql = String::from(&req_body.sql);
    match parse_select(&String::from(sql)) {
        Ok((_, ast)) =>
            HttpResponse::Ok().json(convert_select_statement(&ast)),
        Err(e) => HttpResponse::NoContent().body(e.to_string()),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(todo_index)
    })
        .bind(("web",8080))?
        .run()
        .await
}
