use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;

use actix_web::delete;
use actix_web::get;
use actix_web::post;
use actix_web::web;
use actix_web::Responder;

use crate::db::DatabaseElement;
use crate::shared::dbt as dbt;
use crate::shared::logging::logf;
use crate::shared::req_resp as req;

#[get("/tables")]
pub async fn handler_tables(
    db: web::Data<Arc<Mutex<sled::Db>>>
) -> impl Responder {

    log::info!("{}", logf!("Entered."));

    let db_locked = match db.lock() {
        Ok(lock) => lock,
        Err(_) => {
            log::error!("{}", logf!("Failed to lock database!"));
            return actix_web::HttpResponse::NotFound()
                .body("Database is in a deadlock. (DEADLOCK REFERENCE 😳)")
        }
    };

    let template = dbt::VirtualTable{..Default::default()};
    let tables =
    match template.get_templated(&*db_locked) {
        Ok(tables) => tables,
        Err(_) => {
            log::error!("{}", logf!("Failed to read value from database!"));
            return actix_web::HttpResponse::NotFound()
                .body("Database failed.")
        }
    };
        

    log::info!("{}", logf!("Exited."));

    actix_web::HttpResponse::Ok()
        .json(req::TablesResponseData {tables})

}

#[get("/tables-{id}")]
pub async fn handler_tables_specific(
    user_tables: web::Path<dbt::VirtualTableID>,
    db: web::Data<Arc<Mutex<sled::Db>>>
) -> impl Responder {

    log::info!("{}", logf!("Entered."));

    let db_locked = match db.lock() {
        Ok(lock) => lock,
        Err(_) => {
            log::error!("{}", logf!("Failed to lock database!"));
            return actix_web::HttpResponse::NotFound()
                .body("Database is in a deadlock. (DEADLOCK REFERENCE 😳)")
        }
    };

    match dbt::VirtualTable::get(
        dbt::VirtualTable {
            name: user_tables.into_inner().clone(), 
            ..Default::default()
        }
        .qualified_identifier(),
        &*db_locked
    ) {
        Ok(Some(table)) => {
            log::info!("{}", logf!(format!("Returning {}.", table.name)));
            return actix_web::HttpResponse::Ok()
                .json(req::TablesSpecificResponseData {table})
        },
        Ok(None) => {
            log::info!("{}", logf!("Nothing found."));
            return actix_web::HttpResponse::NotFound()
                .body("Not found.")
        }
        Err(_) => {
            log::error!("{}", logf!("Failed to read value from database."));
            return actix_web::HttpResponse::NotFound()
                .body("Database failed.")
        }
    };

}

#[post("/tables")]
pub async fn handler_tables_insert(
    request_data: web::Json<req::TablesInsertRequestData>,
    db: web::Data<Arc<Mutex<sled::Db>>>
) -> impl Responder { 

    log::info!("{}", logf!("Entered."));

    let db_locked = match db.lock() {
        Ok(lock) => lock,
        Err(_) => {
            log::error!("{}", logf!("Failed to lock database!"));
            return actix_web::HttpResponse::InternalServerError()
                .body("Database is in a deadlock. (DEADLOCK REFERENCE 😳)")
        }
    };

    match request_data.into_inner().table.insert(&db_locked) {
        Ok(()) => {
            return actix_web::HttpResponse::Ok()
                .body("Successfully created the table.")
        }
        Err(err) => {
            log::error!("{}: {}", logf!("Failed to lock database!"), err);
            return actix_web::HttpResponse::InternalServerError()
                .body("Database failed.")
        }
    }

}


#[delete("/tables-{id}")]
pub async fn handler_tables_delete(
    table_id: web::Path<dbt::VirtualTableID>,
    db: web::Data<Arc<Mutex<sled::Db>>>
) -> impl Responder { 

    log::info!("{}", logf!("Entered."));

    let db_locked = match db.lock() {
        Ok(lock) => lock,
        Err(_) => {
            log::error!("{}", logf!("Failed to lock database!"));
            return actix_web::HttpResponse::InternalServerError()
                .body("Database is in a deadlock. (DEADLOCK REFERENCE 😳)")
        }
    };

    let template =  dbt::VirtualTable {
        name: table_id.into_inner(),
        ..Default::default()
    };

    match template.remove(&db_locked){
        Ok(()) => {
            return actix_web::HttpResponse::Ok()
                .body("Successfully removed the table.")         
        }
        Err(err) => {
            log::error!("{}: {}", logf!("Failed to lock database!"), err);
            return actix_web::HttpResponse::InternalServerError()
                .body("Database failed.")
        }
    }

}

#[get("/offers")]
pub async fn handler_offers(
    db: web::Data<Arc<Mutex<sled::Db>>>
) -> impl Responder { 

    log::info!("{}", logf!("Entered."));

    let db_locked = match db.lock() {
        Ok(lock) => lock,
        Err(err) => {
            log::error!("{}: {}", logf!("Failed to lock database!"), err);
            return actix_web::HttpResponse::InternalServerError()
                .body("Database is in a deadlock. (DEADLOCK REFERENCE 😳)")
        }
    };

    let probable_offers = dbt::Offer {
        ..Default::default()
    }.get_templated(&*db_locked);

    let offers = match probable_offers {
        Ok(offers) => offers,
        Err(err) => {
            log::error!("{}: {}", logf!("No offers found!"), err);
            return actix_web::HttpResponse::InternalServerError()
                .body("Failed to get offers due to a database error.")
        } 
    };

    return actix_web::HttpResponse::Ok()
        .json(req::OffersResponseData {offers})

}

#[get("/offers/{id}")]
pub async fn handler_offers_specific(
    user_offer: web::Path<dbt::OfferID>,
    db: web::Data<Arc<Mutex<sled::Db>>>
) -> impl Responder { 

    log::info!("{}", logf!("Entered."));

    let db_locked = match db.lock() {
        Ok(lock) => lock,
        Err(err) => {
            log::error!("{}: {}", logf!("Failed to lock database!"), err);
            return actix_web::HttpResponse::InternalServerError()
                .reason("Database is in a deadlock. (DEADLOCK REFERENCE 😳)")
                .body("")
        }
    };

    match dbt::Offer::get(
        dbt::Offer {
            name: user_offer.into_inner(),
            ..Default::default()
        }.qualified_identifier(), 
        &*db_locked
    ) {
        Ok(Some(offer)) => {
            return actix_web::HttpResponse::Ok()
                .json(req::OffersSpecificResponseData {offer})
        }
        Ok(None) => {
            log::error!("{}", logf!("No offer found!"));
            return actix_web::HttpResponse::NotFound()
                .reason("Does not exist!")
                .body("")
        }
        Err(err) => {
            log::error!("{}: {}", logf!("No offers found!"), err);
            return actix_web::HttpResponse::InternalServerError()
                .reason("Failed to get offers due to a database error.")
                .body("")
        } 
    }

}



#[post("/offers")]
pub async fn handler_offers_insert(
    request_data: web::Json<req::OffersInsertRequestData>,
    db: web::Data<Arc<Mutex<sled::Db>>>
) -> impl Responder { 

    log::info!("{}", logf!("Entered."));

    let db_locked = match db.lock() {
        Ok(lock) => lock,
        Err(_) => {
            log::error!("{}", logf!("Failed to lock database!"));
            return actix_web::HttpResponse::InternalServerError()
                .body("Database is in a deadlock. (DEADLOCK REFERENCE 😳)")
        }
    };

    match request_data.into_inner().offer.insert(&db_locked) {
        Ok(()) => {
            return actix_web::HttpResponse::Ok()
                .body("Successfully created the offer.")
        }
        Err(err) => {
            log::error!("{}: {}", logf!("Failed to lock database!"), err);
            return actix_web::HttpResponse::InternalServerError()
                .body("Database failed.")
        }
    }

}


#[delete("/offers/{id}")]
pub async fn handler_offers_delete(
    offer_id: web::Path<dbt::OfferID>,
    db: web::Data<Arc<Mutex<sled::Db>>>
) -> impl Responder { 

    log::info!("{}", logf!("Entered."));

    let db_locked = match db.lock() {
        Ok(lock) => lock,
        Err(_) => {
            log::error!("{}", logf!("Failed to lock database!"));
            return actix_web::HttpResponse::InternalServerError()
                .body("Database is in a deadlock. (DEADLOCK REFERENCE 😳)")
        }
    };

    let template =  dbt::Offer {
        name: offer_id.into_inner(),
        ..Default::default()
    };

    match template.remove(&db_locked){
        Ok(()) => {
            return actix_web::HttpResponse::Ok()
                .body("Successfully removed the table.")         
        }
        Err(err) => {
            log::error!("{}: {}", logf!("Failed to lock database!"), err);
            return actix_web::HttpResponse::InternalServerError()
                .body("Database failed.")
        }
    }

}

#[get("/orders")]
pub async fn handler_orders(
    data: web::Json<req::OrdersRequestData>,
    db: web::Data<Arc<Mutex<sled::Db>>>
) -> impl Responder { 

    log::info!("{}", logf!("Entered."));

    let db_locked = match db.lock() {
        Ok(lock) => lock,
        Err(err) => {
            log::error!("{}: {}", logf!("Failed to lock database!"), err);
            return actix_web::HttpResponse::InternalServerError()
                .body("Database is in a deadlock. (DEADLOCK REFERENCE 😳)")
        }
    };

    
    let probable_orders = if !data.new && data.table.is_none() {
        dbt::Order::get_all(&*db_locked)
    } else {
        if data.table.is_some() {

            dbt::Order {
                id: dbt::OrderID {
                    count: 0,
                    table: data.table.clone().unwrap()
                },
                finished: !data.new,
                items: vec![]
            }.get_templated(&*db_locked) 

        } else {

            dbt::Order {
                id: dbt::OrderID {
                    count: 0,
                    table: "".to_string()
                },
                finished: !data.new,
                items: vec![]
            }.get_status(&*db_locked) 
            
        }
    };

    let orders = match probable_orders {
        Ok(orders) => orders,
        Err(err) => {
            log::error!("{}: {}", logf!("No offers found!"), err);
            return actix_web::HttpResponse::InternalServerError()
                .body("Failed to get offers due to a database error.")
        } 
    };


    return actix_web::HttpResponse::Ok()
        .json(req::OrdersResponseData {orders})

}

#[get("/orders/specific")]
pub async fn handler_orders_specific(
    data: web::Json<req::OrdersSpecificRequestData>,
    db: web::Data<Arc<Mutex<sled::Db>>>
) -> impl Responder { 

    log::info!("{}", logf!("Entered."));

    let db_locked = match db.lock() {
        Ok(lock) => lock,
        Err(err) => {
            log::error!("{}: {}", logf!("Failed to lock database!"), err);
            return actix_web::HttpResponse::InternalServerError()
                .reason("Database is in a deadlock. (DEADLOCK REFERENCE 😳)")
                .body("")
        }
    };

    match dbt::Order::get(
        data.into_inner().order.qualified_identifier(), 
        &*db_locked
    ) {
        Ok(Some(order)) => {
            return actix_web::HttpResponse::Ok()
                .json(req::OrdersSpecificResponseData {order})
        }
        Ok(None) => {
            log::error!("{}", logf!("No offer found!"));
            return actix_web::HttpResponse::NotFound()
                .reason("Does not exist!")
                .body("")
        }
        Err(err) => {
            log::error!("{}: {}", logf!("No offers found!"), err);
            return actix_web::HttpResponse::InternalServerError()
                .reason("Failed to get offers due to a database error.")
                .body("")
        } 
    }

}



#[post("/orders")]
pub async fn handler_orders_insert(
    data: web::Json<req::OrdersInsertRequestData>,
    db: web::Data<Arc<Mutex<sled::Db>>>
) -> impl Responder { 

    log::info!("{}", logf!("Entered."));

    let db_locked = match db.lock() {
        Ok(lock) => lock,
        Err(_) => {
            log::error!("{}", logf!("Failed to lock database!"));
            return actix_web::HttpResponse::InternalServerError()
                .body("Database is in a deadlock. (DEADLOCK REFERENCE 😳)")
        }
    };

    let mut template = data.into_inner().order;

    let mut table = match dbt::VirtualTable::get(
        dbt::VirtualTable {
            name: template.clone().id.table,
            ..Default::default()
        }.qualified_identifier(), 
        &db_locked
    ) {
        Ok(Some(table)) => table,
        _ => {
            return actix_web::HttpResponse::InternalServerError()
                .body("Database failed.")
        }
    };

    table.order_count = table.order_count+1;
    template.id.count = table.order_count;

    match table.insert(&db_locked) {
        Ok(()) => (),
        _ => {
            return actix_web::HttpResponse::InternalServerError()
                .body("Database failed.")
        }
    }

    match template.insert(&db_locked) {
        Ok(()) => {
            return actix_web::HttpResponse::Ok()
                .body("Successfully created the order.")
        }
        Err(err) => {
            log::error!("{}: {}", logf!("Failed to lock database!"), err);
            return actix_web::HttpResponse::InternalServerError()
                .body("Database failed.")
        }
    }

}


#[delete("/orders")]
pub async fn handler_orders_delete(
    data: web::Json<req::OrdersDeleteRequestData>,
    db: web::Data<Arc<Mutex<sled::Db>>>
) -> impl Responder { 

    log::info!("{}", logf!("Entered."));

    let db_locked = match db.lock() {
        Ok(lock) => lock,
        Err(_) => {
            log::error!("{}", logf!("Failed to lock database!"));
            return actix_web::HttpResponse::InternalServerError()
                .body("Database is in a deadlock. (DEADLOCK REFERENCE 😳)")
        }
    };

    match data.into_inner().order.remove(&db_locked){
        Ok(()) => {
            return actix_web::HttpResponse::Ok()
                .body("Successfully removed the table.")         
        }
        Err(err) => {
            log::error!("{}: {}", logf!("Failed to lock database!"), err);
            return actix_web::HttpResponse::InternalServerError()
                .body("Database failed.")
        }
    }

}

#[post("/orders-finish")]
pub async fn handler_orders_finish(
    data: web::Json<req::OrdersDeleteRequestData>,
    db: web::Data<Arc<Mutex<sled::Db>>>
) -> impl Responder { 

    log::info!("{}", logf!("Entered."));

    let db_locked = match db.lock() {
        Ok(lock) => lock,
        Err(_) => {
            log::error!("{}", logf!("Failed to lock database!"));
            return actix_web::HttpResponse::InternalServerError()
                .body("Database is in a deadlock. (DEADLOCK REFERENCE 😳)")
        }
    };

    let template = data.into_inner().order;

    let order = match dbt::Order::get(
        template.qualified_identifier(),
        &db_locked
    ) {
        Ok(Some(mut order)) => {
            order.finished = true;
            order
        },
        Ok(None) | Err(_) => {
            return actix_web::HttpResponse::InternalServerError()
                .body("Order doesn't exist.")
        }
    };

    match template.remove(&db_locked){
        Ok(()) => (),
        Err(err) => {
            log::error!("{}: {}", logf!("Failed to lock database!"), err);
            return actix_web::HttpResponse::InternalServerError()
                .body("Database failed.")
        }
    }

    match order.insert(&db_locked) {
        Ok(()) => {
            return actix_web::HttpResponse::Ok()
                .json(req::OrdersFinishResponseData {
                    table: order.id.table
                })
        }
        Err(err) => {
            log::error!("{}: {}", logf!("Failed to lock database!"), err);
            return actix_web::HttpResponse::InternalServerError()
                .body("Database failed.")
        }
    }

}


#[get("/offers-tables")]
pub async fn handler_offers_tables(
    db: web::Data<Arc<Mutex<sled::Db>>>
) -> impl Responder { 

    log::info!("{}", logf!("Entered."));

    let db_locked = match db.lock() {
        Ok(lock) => lock,
        Err(_) => {
            log::error!("{}", logf!("Failed to lock database!"));
            return actix_web::HttpResponse::InternalServerError()
                .body("Database is in a deadlock. (DEADLOCK REFERENCE 😳)")
        }
    };

    let tables = 
        match dbt::VirtualTable::get_all(&*db_locked) {
            Ok(tables) => tables,
            Err(_) => return actix_web::HttpResponse::InternalServerError()
                .body("Database failed.")
        };
    
    let offers = 
        match dbt::Offer::get_all(&*db_locked) {
            Ok(orders) => orders,
            Err(_) => return actix_web::HttpResponse::InternalServerError()
                .body("Database failed.")
        };

    return actix_web::HttpResponse::Ok()
        .json(req::OffersTablesResponseData {offers, tables})    
   

}



#[get("/{id}")]
pub async fn handler_server(
    table_id: web::Path<dbt::VirtualTableID>,
    db: web::Data<Arc<Mutex<sled::Db>>>
) -> impl Responder { 

    log::info!("{}", logf!("Entered."));

    let db_locked = match db.lock() {
        Ok(lock) => lock,
        Err(_) => {
            log::error!("{}", logf!("Failed to lock database!"));
            return actix_web::HttpResponse::InternalServerError()
                .body("Database is in a deadlock. (DEADLOCK REFERENCE 😳)")
        }
    };

    let path = vec![
        env!("CARGO_MANIFEST_DIR"), 
        "/index/index.html"
    ].join("/");


    let string = std::fs::read_to_string(path).expect("True");

    let tables = match dbt::VirtualTable::get_all(&db_locked) {
        Ok(tables) => tables,
        Err(_) => return actix_web::HttpResponse::InternalServerError()
            .body("Database failed.")
    };

    let mut set = HashSet::new();
    for table in tables {
        set.insert(table.name.clone());
    }

    if set.contains(&table_id.into_inner()) {
        return actix_web::HttpResponse::Ok()
            .content_type("text/html")
            .body(string)  
    } else {
        return actix_web::HttpResponse::InternalServerError()
            .body("Table does not exist!")
    }


    
}