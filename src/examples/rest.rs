use crate::examples::contactdb::ContactRepository;
use crate::examples::Contact;

use actix_web::{web, HttpRequest, HttpResponse, Responder};

#[allow(dead_code)]
pub async fn get_all() -> impl Responder {
    let repository = ContactRepository;
    match repository.get_all().await {
        Ok(contacts) => HttpResponse::Ok().json(contacts),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[allow(dead_code)]
pub async fn get_by_id(req: HttpRequest) -> impl Responder {
    let repository = ContactRepository;
    let id = req.match_info().get("id").unwrap_or("0").to_string();
    match repository.get_by_id(id).await {
        Ok(contact) => HttpResponse::Ok().json(contact),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[allow(dead_code)]
pub async fn create_contact(contact: web::Json<Contact>) -> impl Responder {
    let repository = ContactRepository;
    match repository.create_contact(contact.into_inner()).await {
        Ok(record) => HttpResponse::Ok().json(record),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::examples::{Contact, Record};
    use actix_http::Request;
    use actix_web::dev::{Service, ServiceResponse};
    use actix_web::web;
    use actix_web::{test, App};

    async fn init_test_app(
    ) -> impl Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
        test::init_service(
            App::new()
                .route("/contacts", web::get().to(get_all))
                .route("/contacts/{id}", web::get().to(get_by_id))
                .route("/contacts", web::post().to(create_contact)),
        )
        .await
    }

    #[tokio::test]
    async fn test_create() {
        let app = init_test_app().await;
        let contact = Contact {
            id: None,
            first: "John".to_string(),
            last: "Doe".to_string(),
            email: Some("jd@abc.com".to_string()),
            phone: Some("1234567890".to_string()),
        };

        let req = test::TestRequest::post()
            .uri("/contacts")
            .set_json(&contact)
            .to_request();

        let resp = test::call_service(&app, req).await;
        if resp.status().is_success() {
            let record: Record = test::read_body_json(resp).await;
            let id_str = record.to_raw_id();
            dbg!(&id_str);
            let req = test::TestRequest::get()
                .uri(format!("/contacts/{}", id_str).as_str())
                .to_request();
            let resp = test::call_service(&app, req).await;
            assert!(resp.status().is_success());

            let contact: Contact = test::read_body_json(resp).await;
            assert_eq!(contact.first, "John");
            dbg!(&contact);
        } else {
            assert!(false);
        }
    }

    #[tokio::test]
    async fn test_get_all() {
        let mut app = init_test_app().await;

        let contact = Contact {
            id: None,
            first: "Jane".to_string(),
            last: "Doe".to_string(),
            email: Some("jane@abc.com".to_string()),
            phone: Some("1234567891".to_string()),
        };

        let req = test::TestRequest::post()
            .uri("/contacts")
            .set_json(&contact)
            .to_request();

        let resp = test::call_service(&app, req).await;
        if resp.status().is_success() {
            let contact = Contact {
                id: None,
                first: "Adam".to_string(),
                last: "Smith".to_string(),
                email: Some("adam@abc.com".to_string()),
                phone: Some("1234567892".to_string()),
            };

            let req = test::TestRequest::post()
                .uri("/contacts")
                .set_json(&contact)
                .to_request();

            let resp = test::call_service(&app, req).await;
            if resp.status().is_success() {
                let req = test::TestRequest::get().uri("/contacts").to_request();
                let resp = test::call_service(&mut app, req).await;
                assert!(resp.status().is_success());
                dbg!(&resp);

                let contacts: Vec<Contact> = test::read_body_json(resp).await;
                dbg!(&contacts);
                // Since this is going to the same DB, there could be other records in the DB.
                let count: i32 = contacts.len() as i32 - 2;
                if count >= 0 {
                    assert!(true);
                } else {
                    assert!(false);
                }
            } else {
                assert!(false);
            }
        } else {
            assert!(false);
        }
    }
}
