use serde::{Deserialize, Serialize};
use crate::shared::dbt as dbt;


/// DatabaseElement is used for everything that needs to be placed
/// inside a `sled` database.
/// 
/// The only functions you need to define are:
/// - [`namespace`](DatabaseElement::namespace)
/// - [`identifier`](DatabaseElement::namespace)
/// - [`status`](DatabaseElement::namespace)
#[allow(dead_code)]
pub trait DatabaseElement: 
    Sized + Serialize +  for<'de> Deserialize<'de> 
{

    const QUALIFIED_SEPARATOR: &'static str = "/";

    /// The unique namespace for the element that allows us
    /// to differentiate different kinds of elements in a database.
    /// 
    /// Example
    /// -------
    /// ```
    /// struct Offer {
    ///     ...
    /// }
    /// 
    /// impl DatabaseElement for Offer {
    ///     fn namespace() -> &'static str {"offer"}
    /// }
    /// ```
    fn namespace() -> &'static str;

    fn self_namespace(&self) -> &'static str {
        Self::namespace()
    }

    /// The main identifier that is completely unique to a instance
    /// of the element that implements this trait.
    /// 
    /// Example
    /// -------
    /// ```
    /// struct Order {
    ///     finished: false
    ///     ...
    /// }
    /// 
    /// impl DatabaseElement for Offer {
    ///     fn main_identifier() -> String {self.name.clone()}
    /// }
    /// ```
    fn main_identifier(&self) -> String;

    /// A secondary identifier that is not unique to a instance of
    /// the element and can be shared among multiple instances.
    /// 
    /// Used mostly for [`get_templated`](DatabaseElement::get_templated).
    fn secondary_identifiers(&self) -> Vec<String>;

    fn secondary_identifiers_try_to_string(&self) -> Option<String> {

        if self.secondary_identifiers().is_empty() {
            None
        } else {
            Some(self.secondary_identifiers().join(Self::QUALIFIED_SEPARATOR))
        }
    }

    /// A status of the element that is viewable from its qualified 
    /// identifier..
    /// 
    /// Usually how it's used is that a specific member in the
    /// elements structure is mapped as a status, so that searching
    /// for elements with a member with some specific value is much
    /// faster.
    /// 
    /// Example
    /// -------
    /// ```
    /// struct Order {
    ///     finished: bool
    ///     ...
    /// }
    /// 
    /// impl DatabaseElement for Offer {
    ///     fn status(&self) -> Vec<String> {
    ///         vec![
    ///             if self.finished {"old"} else {"new"}.into()
    ///         ]
    ///     }
    /// }
    /// ```
    fn status(&self) -> Vec<String>;

    fn status_to_string(&self) -> String {
        format!("({})", self.status().join(","))
    }

    /// The qualified separator with one part missing, the main
    /// identifier.
    /// 
    /// Used for building the actual qualified identifier with
    /// [`qualified_identifier`](DatabaseElement::qualified_identifier)
    /// and searching with [`get_templated`](DatabaseElement::get_templated).
    fn qualified_identifier_mainless(&self) -> String {
        let mut partial = vec![
            self.qualified_identifier_mainless_secondless()
        ];

        self.secondary_identifiers_try_to_string()
            .is_some_and(|x| {partial.push(x); true});

        partial.join(Self::QUALIFIED_SEPARATOR)
    }

        /// The qualified separator with one part missing, the main
    /// identifier.
    /// 
    /// Used for building the actual qualified identifier with
    /// [`qualified_identifier`](DatabaseElement::qualified_identifier)
    /// and searching with [`get_templated`](DatabaseElement::get_templated).
    fn qualified_identifier_mainless_secondless(&self) -> String {
        let mut partial = vec![
            Self::namespace().to_string(),
            self.status_to_string(),
        ];

        partial.join(Self::QUALIFIED_SEPARATOR)
    }



    /// The qualified identifier that is unique to any instance of any
    /// element type inside the database.
    /// 
    /// Example
    /// -------
    /// ```
    /// pub type VirtualTableID = String;
    ///
    /// #[derive(Debug, Serialize, Deserialize, Default, Clone)]
    /// pub struct OrderID {
    ///     pub table: VirtualTableID,
    ///     pub count: u32
    /// }
    ///
    /// #[derive(Debug, Serialize, Deserialize, Default, Clone)]
    /// pub struct OrderItem {
    ///     pub id: OfferID,
    ///     pub count: u32,
    /// }
    /// 
    /// #[derive(Debug, Serialize, Deserialize, Default, Clone)]
    /// pub struct Order {
    ///     pub id: OrderID,
    ///     pub finished: bool,
    ///     pub items: Vec<OrderItem>
    /// }
    /// 
    /// impl DatabaseElement for dbt::Order {
    /// 
    ///     fn namespace() -> &'static str {"order"}
    ///     fn status(&self) -> Vec<String> {
    ///         vec![
    ///             if self.finished {"old"} else {"new"}.into()
    ///         ]
    ///     }
    ///     fn main_identifier(&self) -> String {self.id.count.to_string()}
    ///     fn secondary_identifiers(&self) -> Vec<String> {vec![self.id.table.clone()]}
    /// 
    /// }
    /// 
    /// assert_eq!(
    ///     "order/(new)/table1/689".to_string(),
    ///     dbt::Order {
    ///         id: dbt::OrderID {
    ///             table: "table1",
    ///             count: 689
    ///         },
    ///         finished: false,
    ///         items: vec![]
    ///     }
    /// )
    /// ```
    fn qualified_identifier(&self) -> String {
        vec![
            self.qualified_identifier_mainless(),
            self.main_identifier()
        ].join(Self::QUALIFIED_SEPARATOR)
    }

    fn insert(&self, db: &sled::Db) -> Result<(), String> {
        let serialized = match bincode::serialize(&self) {
            Ok(result) => result,
            Err(err) => return Err(err.to_string())
        };
        match db.insert(self.qualified_identifier(), serialized) {
            Ok(_) => Ok(()),
            Err(err) => Err(err.to_string())
        }
    }

    fn remove(&self, db: &sled::Db) -> Result<(), String> {
        match db.remove(self.qualified_identifier()) {
            Ok(_) => Ok(()),
            Err(err) => Err(err.to_string())
        }
    }

    fn exists(&self, db: &sled::Db) -> Result<bool, String> {
        match db.contains_key(self.qualified_identifier()) {
            Ok(result) => Ok(result),
            Err(err) => Err(err.to_string())
        }
    }

    fn get(id: String, db: &sled::Db) -> Result<Option<Self>, String> {
        let raw_data = match db.get(id) {
            Ok(Some(data)) => data.to_vec(),
            Ok(None) => return Ok(None),
            Err(err) => return Err(err.to_string())

        };

        let deserialize: Self = match bincode::deserialize(&raw_data) {
            Ok(item) => item,
            Err(err) => return Err(err.to_string())
        }; 

        Ok(Some(deserialize))
    }

    fn get_templated(&self, db: &sled::Db) -> Result<Vec<Self>, String> {
        let mut results: Vec<Self> = Vec::new();

        for kv_pair 
        in db.scan_prefix(self.qualified_identifier_mainless()) {

            match kv_pair {
                Ok((_key, sled_raw_value)) => {
                    let raw_value = sled_raw_value.to_vec();
                    let value = match bincode::deserialize::<Self>(
                        &raw_value
                    ) {
                        Ok(value) => value,
                        Err(err) => return Err(err.to_string())
                    };
                    results.push(value)
                }
                Err(_) => {}
            }

        }

        Ok(results)

    }

    fn get_status(&self, db: &sled::Db) -> Result<Vec<Self>, String> {
        let mut results: Vec<Self> = Vec::new();

        for kv_pair 
        in db.scan_prefix(self.qualified_identifier_mainless_secondless()) {

            match kv_pair {
                Ok((_key, sled_raw_value)) => {
                    let raw_value = sled_raw_value.to_vec();
                    let value = match bincode::deserialize::<Self>(
                        &raw_value
                    ) {
                        Ok(value) => value,
                        Err(err) => return Err(err.to_string())
                    };
                    results.push(value)
                }
                Err(_) => {}
            }

        }

        Ok(results)
    }

    fn get_all(db: &sled::Db) -> Result<Vec<Self>, String> {
        let mut results: Vec<Self> = Vec::new();

        for kv_pair 
        in db.scan_prefix(Self::namespace()) {

            match kv_pair {
                Ok((_key, sled_raw_value)) => {
                    let raw_value = sled_raw_value.to_vec();
                    let value = match bincode::deserialize::<Self>(
                        &raw_value
                    ) {
                        Ok(value) => value,
                        Err(err) => return Err(err.to_string())
                    };
                    results.push(value)
                }
                Err(_) => {}
            }

        }

        Ok(results)
    }

}

pub fn database_element_get_kind(s: &str) -> Option<String> {

     match s.split_once('/') {
        Some((kind, _)) => Some(kind.to_string()),
        None => None
    }

}

pub const OFFER_NAMESPACE:         &'static str = "offer";
pub const VIRTUAL_TABLE_NAMESPACE: &'static str = "table";
pub const ORDER_NAMESPACE:         &'static str = "order";

impl DatabaseElement for dbt::Offer {

    fn namespace() -> &'static str {OFFER_NAMESPACE}
    fn status(&self) -> Vec<String> {vec![]}
    fn main_identifier(&self) -> String {self.name.clone()}
    fn secondary_identifiers(&self) -> Vec<String> {vec![]}
   
}

impl DatabaseElement for dbt::VirtualTable {

    fn namespace() -> &'static str {VIRTUAL_TABLE_NAMESPACE}
    fn status(&self) -> Vec<String> {vec![]}
    fn main_identifier(&self) -> String {self.name.clone()}
    fn secondary_identifiers(&self) -> Vec<String> {vec![]}

}

impl DatabaseElement for dbt::Order {

    fn namespace() -> &'static str {ORDER_NAMESPACE}
    fn status(&self) -> Vec<String> {
        vec![
            if self.finished {"old"} else {"new"}.into()
        ]
    }
    fn main_identifier(&self) -> String {self.id.count.to_string()}
    fn secondary_identifiers(&self) -> Vec<String> {vec![self.id.table.clone()]}

}

// impl dbt::Order {

//     pub fn partial_qualified_identifier(&self) -> String {
//         format!("{}/{}/{}", Self::namespace(), self.status_to_string(), self.id.table)
//     }

// }