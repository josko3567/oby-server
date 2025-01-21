use std::collections::HashMap;
use std::task::Poll;
use std::time::Duration;

use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadBuf};

use crate::shared::dbt;

use super::dbt::VirtualTableID;

pub const IP: &str = "192.168.50.118";
pub const DB_PORT: u16 = 8656;
pub const HTML_PORT: u16 = 5000;


pub fn get_local_ip_address() -> Result<String, std::io::Error> {
    match if_addrs::get_if_addrs() {
        Ok(if_addrs) => {
            for iface in if_addrs {
                if let ip = iface.addr.ip() {
                    if ip.is_ipv4() && !ip.is_loopback() {
                        return Ok(ip.to_string());
                    }
                }
            }
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No non-loopback IPv4 address found"))
        }
        Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to get IP addresses: {}", e))),
    }
}

//////////////////////////////////////////////////
// Tables

    #[derive(Serialize, Deserialize)]
    pub struct TablesRequestData;
#[derive(Serialize, Deserialize)]
pub struct TablesResponseData {
    pub tables: Vec<dbt::VirtualTable>
}


    #[derive(Serialize, Deserialize)]
    pub struct TablesSpecificRequestData;
#[derive(Serialize, Deserialize)]
pub struct TablesSpecificResponseData {
    pub table: dbt::VirtualTable
}


    #[derive(Serialize, Deserialize)]
    pub struct TablesInsertRequestData {
        pub table: dbt::VirtualTable
    }
#[derive(Serialize, Deserialize)]
pub struct TablesInsertResponseData;


    #[derive(Serialize, Deserialize)]
    pub struct TablesDeleteRequestData;
#[derive(Serialize, Deserialize)]
pub struct TablesDeleteResponseData;

//////////////////////////////////////////////////
// Offers

    #[derive(Serialize, Deserialize)]
    pub struct OffersRequestData;
#[derive(Serialize, Deserialize)]
pub struct OffersResponseData {
    pub offers: Vec<dbt::Offer>
}


    #[derive(Serialize, Deserialize)]
    pub struct OffersSpecificRequestData;
#[derive(Serialize, Deserialize)]
pub struct OffersSpecificResponseData {
    pub offer: dbt::Offer
}


    #[derive(Serialize, Deserialize)]
    pub struct OffersInsertRequestData {
        pub offer: dbt::Offer
    }
#[derive(Serialize, Deserialize)]
pub struct OffersInsertResponseData;


    #[derive(Serialize, Deserialize)]
    pub struct OffersDeleteRequestData;
#[derive(Serialize, Deserialize)]
pub struct OffersDeleteResponseData;

//////////////////////////////////////////////////
// Orders

    #[derive(Serialize, Deserialize)]
    pub struct OrdersRequestData {
        pub new: bool,
        pub table: Option<VirtualTableID>
    }
#[derive(Serialize, Deserialize)]
pub struct OrdersResponseData {
    pub orders: Vec<dbt::Order>
}


    #[derive(Serialize, Deserialize)]
    pub struct OrdersSpecificRequestData {
        pub order: dbt::Order
    }
#[derive(Serialize, Deserialize)]
pub struct OrdersSpecificResponseData {
    pub order: dbt::Order
}


    #[derive(Serialize, Deserialize)]
    pub struct OrdersInsertRequestData {
        pub order: dbt::Order,
    }
#[derive(Serialize, Deserialize)]
pub struct OrdersInsertResponseData;


    #[derive(Serialize, Deserialize)]
    pub struct OrdersDeleteRequestData {
        pub order: dbt::Order
    }
#[derive(Serialize, Deserialize)]
pub struct OrdersDeleteResponseData;

//////////////////////////////////////////////////
// Custom

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct OffersTablesRequestData;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OffersTablesResponseData {
    pub offers: Vec<dbt::Offer>,
    pub tables: Vec<dbt::VirtualTable>
}


    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct OrdersFinishRequestData {
        pub order: dbt::Order
    }
    #[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrdersFinishResponseData {
    pub table: VirtualTableID
}


pub enum RequestKind {

    Tables,
    TablesSpecific,
    TablesInsert,
    TablesDelete,

    Offers,
    OffersSpecific,
    OffersInsert,
    OffersDelete,

    Orders,
    OrdersSpecific,
    OrdersInsert,
    OrdersDelete,
    OrdersFinish,

    OffersTables,

}

pub struct Request {

    pub kind: RequestKind,
    pub payload: Option<serde_json::Value>

}


pub enum Authority {
    User,
    Admin
}

impl Request {

    pub async fn send_request(&mut self, id: String) -> Result<serde_json::Value, String> {


        log::info!("Entered.");

        // match self.payload.clone() {
        //     Some(payload) => println!("{}", payload.to_string()),
        //     None => println!("No payload")
        // }       
        let LOCAL_IP = get_local_ip_address().expect("Not connected to a network dummy!");

        // let client = Client::new();
        let address = format!("http://{}:{}", LOCAL_IP, DB_PORT);

        const QUERY_ENCODE_SET: &AsciiSet = &CONTROLS
            .add(b' ')
            .add(b'"')
            .add(b'<')
            .add(b'>')
            .add(b'#')
            .add(b'%')
            .add(b'&')
            .add(b'/')
            .add(b'=');

        let (request, body) = match self.kind {

            RequestKind::Tables => {
                let uri = format!("{}/{}",
                    address,
                    utf8_percent_encode(
                        format!(
                            "tables"
                        ).as_str(), 
                        QUERY_ENCODE_SET
                    )
                );
                log::info!("{}", uri);
                (
                    hyper::Request::builder()
                        .method(hyper::Method::GET)
                        .uri(uri)
                        .header("Accept", "*/*")
                        .header("User-Agent", "Rust HTTP Client")
                        .header("Access-Control-Allow-Origin", "*")
                        .body(())
                        .unwrap(),
                    match self.payload.clone() {
                        Some(payload) => payload.to_string(),
                        None => "".to_string()
                    }             
                )               
            }

            RequestKind::TablesSpecific => {
                let uri = format!("{}/{}",
                    address,
                    utf8_percent_encode(
                        format!(
                            "tables-{}",
                            id
                        ).as_str(), 
                        QUERY_ENCODE_SET
                    )
                );
                log::info!("{}", uri);
                (
                    hyper::Request::builder()
                        .method(hyper::Method::GET)
                        .uri(uri)
                        .header("Accept", "*/*")
                        .header("User-Agent", "Rust HTTP Client")
                        .header("Access-Control-Allow-Origin", "*")
                        .body(())
                        .unwrap(),
                    match self.payload.clone() {
                        Some(payload) => payload.to_string(),
                        None => "".to_string()
                    }                
                )                    
            }

            RequestKind::TablesInsert => {
                if self.payload.is_none() {
                    return Err("Payload was empty.".to_string());
                }

                let payload = self.payload.clone().unwrap().to_string();
                let payload_length = payload.len();
                let uri = format!("{}/{}",
                    address,
                    utf8_percent_encode(
                        format!(
                            "tables"
                        ).as_str(), 
                        QUERY_ENCODE_SET
                    )
                );
                log::info!("{} -> {}", uri, payload);
                (
                    hyper::Request::builder()
                        .method(hyper::Method::POST)
                        .uri(uri)
                        .header("Accept", "*/*")
                        .header("Content-Type", "application/json")
                        .header("Content-Length", payload_length.to_string())
                        .header("Access-Control-Allow-Origin", "*")
                        .header("User-Agent", "Rust HTTP Client")
                        .body(())
                        .unwrap(),
                    payload
                )                    
            }

            RequestKind::TablesDelete => {
                let uri = format!("{}/{}",
                    address,
                    utf8_percent_encode(
                        format!(
                            "tables-{}",
                            id
                        ).as_str(), 
                        QUERY_ENCODE_SET
                    )
                );
                log::info!("{}", uri);
                (
                    hyper::Request::builder()
                        .method(hyper::Method::DELETE)
                        .uri(uri)
                        .header("Accept", "*/*")
                        .header("User-Agent", "Rust HTTP Client")
                        .header("Access-Control-Allow-Origin", "*")
                        .body(())
                        .unwrap(),
                    match self.payload.clone() {
                        Some(payload) => payload.to_string(),
                        None => "".to_string()
                    }                
                )                    
            }

            RequestKind::Offers => {
                let uri = format!("{}/{}",
                    address,
                    utf8_percent_encode(
                        format!(
                            "offers"
                        ).as_str(), 
                        QUERY_ENCODE_SET
                    )
                );
                log::info!("{}", uri);
                (
                    hyper::Request::builder()
                        .method(hyper::Method::GET)
                        .uri(uri)
                        .header("Accept", "*/*")
                        .header("User-Agent", "Rust HTTP Client")
                        .header("Access-Control-Allow-Origin", "*")
                        .body(())
                        .unwrap(),
                    match self.payload.clone() {
                        Some(payload) => payload.to_string(),
                        None => "".to_string()
                    }             
                )               
            }

            RequestKind::OffersSpecific => {
                let uri = format!("{}/{}",
                    address,
                    utf8_percent_encode(
                        format!(
                            "offers/{}",
                            id
                        ).as_str(), 
                        QUERY_ENCODE_SET
                    )
                );
                log::info!("{}", uri);
                (
                    hyper::Request::builder()
                        .method(hyper::Method::GET)
                        .uri(uri)
                        .header("Accept", "*/*")
                        .header("User-Agent", "Rust HTTP Client")
                        .header("Access-Control-Allow-Origin", "*")
                        .body(())
                        .unwrap(),
                    match self.payload.clone() {
                        Some(payload) => payload.to_string(),
                        None => "".to_string()
                    }                
                )                    
            }

            RequestKind::OffersInsert => {
                if self.payload.is_none() {
                    return Err("Payload was empty.".to_string());
                }

                let payload = self.payload.clone().unwrap().to_string();
                let payload_length = payload.len();
                let uri = format!("{}/{}",
                    address,
                    utf8_percent_encode(
                        format!(
                            "offers"
                        ).as_str(), 
                        QUERY_ENCODE_SET
                    )
                );
                log::info!("{} -> {}", uri, payload);
                (
                    hyper::Request::builder()
                        .method(hyper::Method::POST)
                        .uri(uri)
                        .header("Accept", "*/*")
                        .header("Content-Type", "application/json")
                        .header("Content-Length", payload_length.to_string())
                        .header("Access-Control-Allow-Origin", "*")
                        .header("User-Agent", "Rust HTTP Client")
                        .body(())
                        .unwrap(),
                    payload
                )                    
            }

            RequestKind::OffersDelete => {
                let uri = format!("{}/{}",
                    address,
                    utf8_percent_encode(
                        format!(
                            "offers/{}",
                            id
                        ).as_str(), 
                        QUERY_ENCODE_SET
                    )
                );
                log::info!("{}", uri);
                (
                    hyper::Request::builder()
                        .method(hyper::Method::DELETE)
                        .uri(uri)
                        .header("Accept", "*/*")
                        .header("User-Agent", "Rust HTTP Client")
                        .header("Access-Control-Allow-Origin", "*")
                        .body(())
                        .unwrap(),
                    match self.payload.clone() {
                        Some(payload) => payload.to_string(),
                        None => "".to_string()
                    }                
                )                    
            }

            RequestKind::Orders => {
                if self.payload.is_none() {
                    return Err("Payload was empty.".to_string());
                }

                let payload = self.payload.clone().unwrap().to_string();
                let payload_length = payload.len();
                
                let uri = format!("{}/{}",
                    address,
                    utf8_percent_encode(
                        format!(
                            "orders"
                        ).as_str(), 
                        QUERY_ENCODE_SET
                    )
                );
                log::info!("{}", uri);
                (
                    hyper::Request::builder()
                        .method(hyper::Method::GET)
                        .uri(uri)
                        .header("Accept", "*/*")
                        .header("Content-Type", "application/json")
                        .header("Content-Length", payload_length.to_string())
                        .header("Access-Control-Allow-Origin", "*")
                        .header("User-Agent", "Rust HTTP Client")
                        .body(())
                        .unwrap(),
                    payload         
                )               
            }

            RequestKind::OrdersSpecific => {
                if self.payload.is_none() {
                    return Err("Payload was empty.".to_string());
                }

                let payload = self.payload.clone().unwrap().to_string();
                let payload_length = payload.len();
                let uri = format!("{}/{}",
                    address,
                    utf8_percent_encode(
                        format!(
                            "orders/specific"
                        ).as_str(), 
                        QUERY_ENCODE_SET
                    )
                );
                log::info!("{} -> {}", uri, payload);
                (
                    hyper::Request::builder()
                        .method(hyper::Method::GET)
                        .uri(uri)
                        .header("Accept", "*/*")
                        .header("Content-Type", "application/json")
                        .header("Content-Length", payload_length.to_string())
                        .header("Access-Control-Allow-Origin", "*")
                        .header("User-Agent", "Rust HTTP Client")
                        .body(())
                        .unwrap(),
                    payload
                )                              
            }

            RequestKind::OrdersInsert => {
                if self.payload.is_none() {
                    return Err("Payload was empty.".to_string());
                }

                let payload = self.payload.clone().unwrap().to_string();
                let payload_length = payload.len();
                let uri = format!("{}/{}",
                    address,
                    utf8_percent_encode(
                        format!(
                            "orders"
                        ).as_str(), 
                        QUERY_ENCODE_SET
                    )
                );
                log::info!("{} -> {}", uri, payload);
                (
                    hyper::Request::builder()
                        .method(hyper::Method::POST)
                        .uri(uri)
                        .header("Accept", "*/*")
                        .header("Content-Type", "application/json")
                        .header("Content-Length", payload_length.to_string())
                        .header("Access-Control-Allow-Origin", "*")
                        .header("User-Agent", "Rust HTTP Client")
                        .body(())
                        .unwrap(),
                    payload
                )                    
            }

            RequestKind::OrdersDelete => {
                if self.payload.is_none() {
                    return Err("Payload was empty.".to_string());
                }

                let payload = self.payload.clone().unwrap().to_string();
                let payload_length = payload.len();
                let uri = format!("{}/{}",
                    address,
                    utf8_percent_encode(
                        format!(
                            "orders"
                        ).as_str(), 
                        QUERY_ENCODE_SET
                    )
                );
                let uri = format!("{}/{}",
                    address,
                    utf8_percent_encode(
                        format!(
                            "orders",
                        ).as_str(), 
                        QUERY_ENCODE_SET
                    )
                );
                log::info!("{}", uri);
                (
                    hyper::Request::builder()
                        .method(hyper::Method::DELETE)
                        .uri(uri)
                        .header("Accept", "*/*")
                        .header("Content-Type", "application/json")
                        .header("Access-Control-Allow-Origin", "*")
                        .header("Content-Length", payload_length.to_string())
                        .header("User-Agent", "Rust HTTP Client")
                        .body(())
                        .unwrap(),
                    payload               
                )                    
            }

            RequestKind::OffersTables => {
 ;
                let uri = format!("{}/{}",
                    address,
                    utf8_percent_encode(
                        format!(
                            "offers-tables"
                        ).as_str(), 
                        QUERY_ENCODE_SET
                    )
                );
                log::info!("{}", uri);
                (
                    hyper::Request::builder()
                        .method(hyper::Method::GET)
                        .uri(uri)
                        .header("Accept", "*/*")
                        .header("Access-Control-Allow-Origin", "*")
                        .header("User-Agent", "Rust HTTP Client")
                        .body(())
                        .unwrap(),
                    match self.payload.clone() {
                        Some(payload) => payload.to_string(),
                        None => "".to_string()
                    }   
                )                    
            }

            RequestKind::OrdersFinish => {
                if self.payload.is_none() {
                    return Err("Payload was empty.".to_string());
                }

                let payload = self.payload.clone().unwrap().to_string();
                let payload_length = payload.len();
                let uri = format!("{}/{}",
                    address,
                    utf8_percent_encode(
                        format!(
                            "orders-finish"
                        ).as_str(), 
                        QUERY_ENCODE_SET
                    )
                );
                log::info!("{} -> {}", uri, payload);
                (
                    hyper::Request::builder()
                        .method(hyper::Method::POST)
                        .uri(uri)
                        .header("Accept", "*/*")
                        .header("Content-Type", "application/json")
                        .header("Access-Control-Allow-Origin", "*")
                        .header("Content-Length", payload_length.to_string())
                        .header("User-Agent", "Rust HTTP Client")
                        .body(())
                        .unwrap(),
                    payload
                )  
            }

            // _ => (
            //     hyper::Request::builder()
            //         .method(hyper::Method::GET)
            //         .uri(format!("{}/", address))
            //         .header("User-Agent", "Rust HTTP Client")
            //         .body(())
            //         .unwrap(),
            //     "".to_string()
            // )  
        };
        
        let request_bytes = format!(
            "{} {} {}\r\n{}\r\n\r\n{}",
            request.method(),
            request.uri().path(),
            "HTTP/1.1",
            request.headers()
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v.to_str().unwrap()))
                .collect::<Vec<String>>()
                .join("\r\n"),
            body
        );

        log::info!("{:#?}", request_bytes);

        let mut stream = match  TcpStream::connect(
            format!("{}:{}", LOCAL_IP.clone(), DB_PORT)
        ).await {

            Ok(stream) => stream,
            Err(err) => return Err(err.to_string())
     
        };

        stream.set_nodelay(true).expect("Failed to set no_delay");

        stream.write(request_bytes.as_bytes()).await.expect("FUCK");

        const BUFFER_SIZE: usize = 16000;
        let mut buffer = [0; BUFFER_SIZE];
        let mut response_bytes = Vec::new();
        let mut total_bytes_read = 0;
        let mut headers_received = false;

        loop {
            let bytes_read = stream.read(&mut buffer).await.expect("Cannot read.");

            if bytes_read == 0 {
                break; // Connection closed by the server
            }

            response_bytes.extend_from_slice(&buffer[..bytes_read]);
            total_bytes_read += bytes_read;

            if !headers_received {
                if let Some(pos) = response_bytes.windows(4).position(|window| window == b"\r\n\r\n") {
                    headers_received = true;
                    let headers = &response_bytes[..pos + 4];
                    log::info!("Headers: {:?}", String::from_utf8_lossy(headers));
                    log::info!("{:?}", String::from_utf8(response_bytes.clone()));
                }
            }

            // let mut read_buf = ReadBuf::new(&mut buffer);

            if bytes_read != BUFFER_SIZE {
                break;
            }

            // match std::future::poll_fn(|cx|
            //     stream.poll_peek(cx, &mut read_buf)
            // ).await {
            //     Ok(size) => {
            //         if size != 0 {
            //             continue;
            //         }
            //         break;
            //     }
            //     Err(err) => return Err(err.to_string())
            // }

        }

        let response_body = String::from_utf8_lossy(&response_bytes);


        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut response = httparse::Response::new(&mut headers);

        let result = response.parse(response_body.as_bytes());

        match result {
            Ok(httparse::Status::Complete(body_offset)) => {
                let body = &response_body[body_offset..];
                return match self.kind {
                    RequestKind::Tables         |
                    RequestKind::TablesSpecific |
                    RequestKind::Offers         |
                    RequestKind::OffersSpecific | 
                    RequestKind::Orders         |
                    RequestKind::OrdersSpecific |
                    RequestKind::OrdersFinish   |
                    RequestKind::OffersTables 
                    => {
                        log::info!("We outta here!");
                        match serde_json::from_str(body) {
                            Ok(result) => {
                                log::info!("Exit. Contains body.");
                                Ok(result)
                            },
                            Err(err) => {
                                log::error!("Error Exit. Borked body.");
                                Err("Invalid data received. Did you perhaps change the database elements?".to_string())
                            }
                        }
                    }
                    _ => {
                        log::info!("No body!");
                        Ok(serde_json::Value::Null.into())
                    }
                }
            }
            _ => {
                log::error!("Error exit. Borked http response.");
                return Err("Server side issue. Did you perhaps turn the server on?".to_string())
            }
        }


        // match response {
        //     Ok(inner) => {
        //         if inner.status().is_success() {
        //             let text = match inner.text().await {
        //                 Ok(text) => text,
        //                 Err(err) => {
        //                     return Err(err.to_string())
        //                 }
        //             };
        //             Ok(json!(text))
        //         } else {
        //             Err(inner.status().as_str().to_string())
        //         }
        //     }
        //     Err(err) => Err(err.to_string())
        // }
        
    }

}
