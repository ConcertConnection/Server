use crate::database::{DatabaseConnection, UserConditions};
use crate::database::User;
use actix_web::{HttpResponse, web};

pub async fn get_user(user_id: web::Form<UserConditions>, database_connection: &DatabaseConnection) -> HttpResponse {


    let query_results = database_connection.execute::<User, UserConditions>(
        &String::from("user_table"),
        &mut String::from("select"),
        None,
        Some(&user_id)
    )
        .await;
    let mut query_results = match query_results {
        Ok(result) => result.rows_typed::<User>().unwrap(),
        Err(err) => return HttpResponse::BadRequest().finish()
    };

    let user = query_results.next().unwrap().unwrap();
    let response = HttpResponse::Ok().json(user);
    response

}