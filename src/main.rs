mod db;
mod shared;
mod requests_database;

use std::{net::TcpStream, path::PathBuf, sync::{Arc, Mutex}};

use actix_web::{web, App, HttpServer};
use db::DatabaseElement;
use shared::dbt as dbt;
use shared::req_resp as req;

pub fn print_db(db: &sled::Db) {
    for kv in db.iter() {

        match kv {
            Ok((key, value)) => {
                let (key_own, value_own) = (key.to_vec(), value.to_vec());
                let key_str = std::str::from_utf8(&key_own).unwrap();
                let key_kind = db::database_element_get_kind(key_str).expect("Failed to convert `key_str` into `key_kind`");
                eprint!("`{}` of type ", key_str);
                match key_kind.as_str() {
                    db::OFFER_NAMESPACE => {
                        let value = bincode::deserialize::<dbt::Offer>(&value_own)
                            .expect(format!("Failed to convert a suspected `{}` into its type.", db::OFFER_NAMESPACE ).as_str());
                        eprintln!("{:#?}\n", value);
                    },
                    db::VIRTUAL_TABLE_NAMESPACE => {
                        let value = bincode::deserialize::<dbt::VirtualTable>(&value_own)
                            .expect(format!("Failed to convert a suspected `{}` into its type.", db::VIRTUAL_TABLE_NAMESPACE ).as_str());
                        eprintln!("{:#?}\n", value);
                    },
                    db::ORDER_NAMESPACE => {
                        let value = bincode::deserialize::<dbt::Order>(&value_own)
                            .expect(format!("Failed to convert a suspected `{}` into its type.", db::VIRTUAL_TABLE_NAMESPACE ).as_str());
                        eprintln!("{:#?}\n", value);
                    }
                    _ => {eprintln!("Unknown key kind?")}
                }
            }
            Err(_) => {eprintln!("\t![Something went wrong]")}
        }

    }
}

fn summon_db() -> Arc<Mutex<sled::Db>> {

    Arc::new(
        Mutex::new(
            sled::open(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("database")
                    .join("regular.sled")
            )
            .expect("Failed to load database: ")
        )
    )

}

fn fill_db(db: &sled::Db) {

    vec![
        dbt::Offer {
            name: "Mala Kava".to_string(),
            description: "Mala kava od sviježi sjemenki.".to_string(),
            price_integer: 1,
            price_fraction: 50
        },
        dbt::Offer {
            name: "Cedevita".to_string(),
            description: "Osvježavajuće piće.".to_string(),
            price_integer: 2,
            price_fraction: 40
        }
    ].into_iter().for_each(|x|{
        x
        .insert(db)
        .expect(
            format!(
                    "Failed to insert element of type `{}` and id `{}`", 
                    x.self_namespace(),
                    x.main_identifier()
                ).as_str()
            );
    });

    vec![
        dbt::VirtualTable {
            name: "Stol 1".to_string(),
            order_count: 0,
        },
        dbt::VirtualTable {
            name: "Stol 2".to_string(),
            order_count: 3,
        },
        dbt::VirtualTable {
            name: "Stol 3".to_string(),
            order_count: 2,
        },
        dbt::VirtualTable {
            name: "Stol 4".to_string(),
            order_count: 5,
        },
        dbt::VirtualTable {
            name: "Stol 5".to_string(),
            order_count: 7,
        }
    ].into_iter().for_each(|x|{
        x
        .insert(db)
        .expect(
            format!(
                    "Failed to insert element of type `{}` and id `{}`", 
                    x.self_namespace(),
                    x.main_identifier()
                ).as_str()
            );
    });

    vec![
        dbt::Order {
            id: dbt::OrderID {
                table: "Stol 2".to_string(),
                count: 1,
            },
            finished: false,
            items: vec![
                dbt::OrderItem {id: "Kava".to_string(), count: 2},
                dbt::OrderItem {id: "Cedevita".to_string(), count: 4}
            ]
        },
        dbt::Order {
            id: dbt::OrderID {
                table: "Stol 1".to_string(),
                count: 2,
            },
            finished: false,
            items: vec![
                dbt::OrderItem {id: "Kava".to_string(), count: 1},
            ]
        },
        dbt::Order {
            id: dbt::OrderID {
                table: "Stol 3".to_string(),
                count: 3,
            },
            finished: false,
            items: vec![
                dbt::OrderItem {id: "Kava".to_string(), count: 3},
                dbt::OrderItem {id: "Cedevita".to_string(), count: 2}
            ]
        },
        dbt::Order {
            id: dbt::OrderID {
                table: "Stol 5".to_string(),
                count: 4,
            },
            finished: false,
            items: vec![
                dbt::OrderItem {id: "Kava".to_string(), count: 5},
                dbt::OrderItem {id: "Cedevita".to_string(), count: 4}
            ]
        }
    ].into_iter().for_each(|x|{
        x
        .insert(db)
        .expect(
            format!(
                    "Failed to insert element of type `{}` and id `{}`", 
                    x.self_namespace(),
                    x.main_identifier()
                ).as_str()
            );
    });

}

#[macro_export]
macro_rules! log_thread_info {
    ($id:ident, $s:literal $(,)? $($arg:tt) ,*)
    =>
    (log::info!(concat!("[Thread {}] ", $s), $id, $($arg)*));
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    env_logger::init();

    log::info!("Summoning database...");
    let db = summon_db();
    log::info!("Database summoned.");
    
    {
        let db_safe = db.lock().expect("Cannot lock database.");
        fill_db(&*db_safe);
    }

    // let server_db = 
    HttpServer::new(move || 
        {
            App::new()
                .wrap(actix_web::middleware::Logger::default())
                .wrap(actix_web::middleware::Logger::new("%a %r"))
                .app_data(web::Data::new(db.clone()))

                .service(requests_database::handler_tables)
                .service(requests_database::handler_tables_specific)
                .service(requests_database::handler_tables_insert)
                .service(requests_database::handler_tables_delete)

                .service(requests_database::handler_offers)
                .service(requests_database::handler_offers_specific)
                .service(requests_database::handler_offers_insert)
                .service(requests_database::handler_offers_delete)

        }
    )
    .bind(("127.0.0.1", 8656))?
    .run()
    .await

}

