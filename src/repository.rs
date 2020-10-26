use serde::{Deserialize, Serialize};
use bson::{doc,oid::ObjectId, Bson};
use mongodb::{options::ClientOptions, options::FindOptions, Client};
use actix_web::{web};
use std::sync::Mutex;
use futures::stream::StreamExt;
use async_recursion::async_recursion;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {    
    #[serde(rename = "_id")]
    pub id: ObjectId, 
    pub name: String,  
    pub age: isize,
}

#[derive(Deserialize, Serialize)]
pub struct NewUser {
    pub name: String,
    pub age: i32,
}

async fn get_collection() -> mongodb::Collection {
    let client_options = ClientOptions::parse("mongodb://localhost:27017").await.unwrap();
    let client = web::Data::new(Mutex::new(Client::with_options(client_options).unwrap()));
    let coll = client
        .lock()
        .unwrap()
        .database("test1")
        .collection("users");
    return coll;   
}

pub async fn get_sort_users() -> Result<Vec<User>, String> {
    let coll = get_collection().await;

    let filter = doc! {};
    let find_options = FindOptions::builder().sort(doc! { "_id": -1}).build();
    let mut cursor = coll.find(filter, find_options).await.unwrap();

    return rec_get_users(&mut cursor, None).await;
}

#[async_recursion]
async fn rec_get_users(cur: &mut mongodb::Cursor, vec_option: Option<Vec<User>>) -> Result<Vec<User>, String> {
    let vec : Vec<User> = vec_option.unwrap_or(Vec::new());
    if let Some(result) = cur.next().await {
        match result {
            Ok(document) => {
                let user : User = bson::from_bson(Bson::Document(document)).unwrap();
                let new_vec = [vec![user], vec].concat();
                return rec_get_users(cur, Some(new_vec)).await;
            }
            _ => {
                return Err(String::from("Server error"));
            }
        }
    }
    else {
        return Ok(vec);
    }
}

pub async fn get_user_by_id(id: &str) -> Option<User> {
    let coll = get_collection().await;
    let obj_id = bson::oid::ObjectId::with_string(id);
    match &obj_id {
        Ok(_) => {
        }
        Err(_) => {
            return None;
        }
    }

    let result = coll.find_one(Some(doc! { "_id": obj_id.unwrap() }), None).await;

    match result {
        Ok(document) => {
            let user : User = bson::from_bson(Bson::Document(document.unwrap())).unwrap();
            return Some(user);
        }
        _ => {
            return None;
        }
    }
}

pub async fn create_user(new_user: NewUser) -> Result<bson::Bson,()> {
    let coll = get_collection().await;
        
    let user_bson = bson::to_bson(&new_user).unwrap();
    let doc = user_bson.as_document().unwrap().clone();
    match coll.insert_one(doc, None).await {
        Ok(db_result) => {
            if let Some(new_id) = db_result.inserted_id.as_object_id() {
                println!("New document inserted with id {}", new_id);   
            }
            return Ok(db_result.inserted_id)
        }
        Err(err) =>
        {
            println!("Failed! {}", err);
            return Err(())
        }
    }
}