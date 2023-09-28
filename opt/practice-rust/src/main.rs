use actix_web::{get, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};


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


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(todo_index)
    })
        .bind(("web","8080"))?
        .run()
        .await
}
