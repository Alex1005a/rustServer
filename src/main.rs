use actix_web::{web, App, HttpResponse, HttpServer, Responder};
mod repository;


async fn get_users() -> impl Responder {
    let result = repository::get_sort_users().await;
    match result {
        Ok(users) => {
            HttpResponse::Ok().json(users)
        }
        _ => {
            return HttpResponse::InternalServerError().finish();
        }
    }
}
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

async fn get_user(info: web::Path<(String,)>) -> impl Responder {
    let result = repository::get_user_by_id(&info.0).await;

    match result {
        Some(user) => {
            HttpResponse::Ok().json(user)
        }
        None => {
            HttpResponse::InternalServerError().finish()
        }
    }
    
}

async fn create_user(new_user: web::Json<repository::NewUser>) -> impl Responder {
    let result = repository::create_user(new_user.into_inner()).await;

    match result {
        Ok(db_result) => {
            return HttpResponse::Created().json(db_result)
        }
        Err(_) =>
        {
            return HttpResponse::InternalServerError().finish()
        }
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/users", web::get().to(get_users))
            .route("/users", web::post().to(create_user))
            .route("/users/{id}", web::get().to(get_user))
    })
    .bind("127.0.0.1:8088")?
    .run()
    .await
}