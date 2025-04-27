use actix_web::{web, HttpResponse, Responder};
use actix_session::Session;

use crate::{schema::{LoginForm, RegisterForm}, services::user::UserService};

pub async fn login(
    form: web::Form<LoginForm>,
    session: Session,
    user_service: web::Data<UserService>,
) -> impl Responder {
    let form = form.into_inner();
    match user_service.authenticate(&form.email, form.password).await {
        Ok(user) => {
            if let Err(e) = session.insert("user_id", user.id) {
                tracing::error!("Failed to insert user_id into session: {:?}", e);
                return HttpResponse::InternalServerError().finish();
            }
            
            if let Ok(Some(return_to)) = session.get::<String>("return_to") {
                session.remove("return_to");
                
                return HttpResponse::Found()
                    .append_header(("Location", return_to))
                    .finish();
            }
            
            HttpResponse::Found()
                .append_header(("Location", "/auth/success"))
                .finish()
        },
        Err(_) => {
            HttpResponse::Found()
                .append_header(("Location", "/login?login_error=Неверный+email+или+пароль"))
                .finish()
        }
    }
}

pub async fn register(
    form: web::Form<RegisterForm>,
    user_service: web::Data<UserService>,
) -> impl Responder {
    if form.password != form.password_confirm {
        return HttpResponse::Found()
            .append_header(("Location", "/login?register_error=Пароли+не+совпадают"))
            .finish();
    }
    
    match user_service.register(&form.name, &form.email, &form.password).await {
        Ok(_) => {
            HttpResponse::Found()
                .append_header(("Location", "/login?registered=true"))
                .finish()
        },
        Err(e) => {
            tracing::error!("Failed to register user: {:?}", e);
            HttpResponse::Found()
                .append_header(("Location", "/login?register_error=Ошибка+сервера+при+регистрации"))
                .finish()
        }
    }
}

// TODO: move to a separate html file
pub async fn success_page() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Успешная авторизация</title>
                <style>
                    body {
                        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
                        background-color: #f7f9fc;
                        display: flex;
                        justify-content: center;
                        align-items: center;
                        height: 100vh;
                        margin: 0;
                    }
                    .success-container {
                        background: white;
                        border-radius: 8px;
                        box-shadow: 0 4px 16px rgba(0,0,0,0.1);
                        padding: 40px;
                        text-align: center;
                        max-width: 500px;
                    }
                    h1 {
                        color: #4361ee;
                        margin-top: 0;
                    }
                    .icon {
                        width: 80px;
                        height: 80px;
                        margin-bottom: 20px;
                        fill: #4361ee;
                    }
                    p {
                        color: #4a5568;
                        line-height: 1.6;
                    }
                    .note {
                        margin-top: 30px;
                        padding: 12px;
                        background: #f1f5f9;
                        border-radius: 4px;
                        font-size: 0.9em;
                    }
                </style>
            </head>
            <body>
                <div class="success-container">
                    <svg class="icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                        <path d="M12,0A12,12,0,1,0,24,12,12,12,0,0,0,12,0Zm6.93,8.2-6.85,9.29a1,1,0,0,1-1.43.19L5.76,13.77a1,1,0,0,1-.15-1.41,1.06,1.06,0,0,1,1.41-.15l4.08,3.22L17.58,7a1,1,0,0,1,1.4-.05A1,1,0,0,1,18.93,8.2Z"/>
                    </svg>
                    <h1>Авторизация успешна</h1>
                    <p>Эта страница отображается, поскольку вы вошли напрямую в сервис аутентификации.</p>
                    <div class="note">
                        <p>Примечание: Обычно пользователи не видят эту страницу, поскольку они перенаправляются обратно в приложение, которое запросило авторизацию.</p>
                    </div>
                </div>
            </body>
            </html>
        "#)
}


pub fn configure() -> impl FnOnce(&mut web::ServiceConfig) {
    |config| {
        config
            .route("/auth/login", web::post().to(login))
            .route("/auth/register", web::post().to(register))
            .route("/auth/success", web::get().to(success_page));
    }
}