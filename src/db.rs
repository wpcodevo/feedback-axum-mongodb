use crate::error::MyError;
use crate::response::{
    FeedbackData, FeedbackListResponse, FeedbackResponse, SingleFeedbackResponse,
};
use crate::{
    error::MyError::*, model::FeedbackModel, schema::CreateFeedbackSchema,
    schema::UpdateFeedbackSchema,
};
use chrono::prelude::*;
use futures::StreamExt;
use mongodb::bson::{doc, oid::ObjectId, Document};
use mongodb::options::{IndexOptions, ReturnDocument};
use mongodb::{bson, options::ClientOptions, Client, Collection, IndexModel};
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct DB {
    pub feedback_collection: Collection<FeedbackModel>,
    pub collection: Collection<Document>,
}

type Result<T> = std::result::Result<T, MyError>;

impl DB {
    pub async fn init() -> Result<Self> {
        let mongodb_uri = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
        let database_name =
            std::env::var("MONGO_INITDB_DATABASE").expect("MONGO_INITDB_DATABASE must be set.");
        let collection_name =
            std::env::var("MONGODB_NOTE_COLLECTION").expect("MONGODB_NOTE_COLLECTION must be set.");

        let mut client_options = ClientOptions::parse(mongodb_uri).await?;
        client_options.app_name = Some(database_name.to_string());

        let client = Client::with_options(client_options)?;
        let database = client.database(database_name.as_str());

        let feedback_collection = database.collection(collection_name.as_str());
        let collection = database.collection::<Document>(collection_name.as_str());

        println!("✅ Database connected successfully");

        Ok(Self {
            feedback_collection,
            collection,
        })
    }

    pub async fn fetch_feedbacks(&self, limit: i64, page: i64) -> Result<FeedbackListResponse> {
        let mut cursor = self
            .feedback_collection
            .find(doc! {})
            .limit(limit)
            .skip(u64::try_from((page - 1) * limit).unwrap())
            .await
            .map_err(MongoQueryError)?;

        let mut json_result: Vec<FeedbackResponse> = Vec::new();
        while let Some(doc) = cursor.next().await {
            json_result.push(self.doc_to_feedback(&doc.unwrap())?);
        }

        Ok(FeedbackListResponse {
            status: "success",
            results: json_result.len(),
            feedbacks: json_result,
        })
    }

    pub async fn create_feedback(
        &self,
        body: &CreateFeedbackSchema,
    ) -> Result<SingleFeedbackResponse> {
        let status = String::from("pending");

        let document = self.create_feedback_document(body, status)?;

        let options = IndexOptions::builder().unique(true).build();
        let index = IndexModel::builder()
            .keys(doc! {"feedback": 1})
            .options(options)
            .build();

        match self.feedback_collection.create_index(index).await {
            Ok(_) => {}
            Err(e) => return Err(MongoQueryError(e)),
        };

        let insert_result = match self.collection.insert_one(&document).await {
            Ok(result) => result,
            Err(e) => {
                if e.to_string()
                    .contains("E11000 duplicate key error collection")
                {
                    return Err(MongoDuplicateError(e));
                }
                return Err(MongoQueryError(e));
            }
        };

        let new_id = insert_result
            .inserted_id
            .as_object_id()
            .expect("issue with new _id");

        let feedback_doc = match self
            .feedback_collection
            .find_one(doc! {"_id": new_id})
            .await
        {
            Ok(Some(doc)) => doc,
            Ok(None) => return Err(NotFoundError(new_id.to_string())),
            Err(e) => return Err(MongoQueryError(e)),
        };

        Ok(SingleFeedbackResponse {
            status: "success",
            data: FeedbackData {
                feedback: self.doc_to_feedback(&feedback_doc)?,
            },
        })
    }

    pub async fn get_feedback(&self, id: &str) -> Result<SingleFeedbackResponse> {
        let oid = ObjectId::from_str(id).map_err(|_| InvalidIDError(id.to_owned()))?;

        let feedback_doc = self
            .feedback_collection
            .find_one(doc! {"_id":oid })
            .await
            .map_err(MongoQueryError)?;

        match feedback_doc {
            Some(doc) => {
                let feedback = self.doc_to_feedback(&doc)?;
                Ok(SingleFeedbackResponse {
                    status: "success",
                    data: FeedbackData { feedback },
                })
            }
            None => Err(NotFoundError(id.to_string())),
        }
    }

    pub async fn edit_feedback(
        &self,
        id: &str,
        body: &UpdateFeedbackSchema,
    ) -> Result<SingleFeedbackResponse> {
        let oid = ObjectId::from_str(id).map_err(|_| InvalidIDError(id.to_owned()))?;

        let update = doc! {
            "$set": bson::to_document(body).map_err(MongoSerializeBsonError)?,
        };

        if let Some(doc) = self
            .feedback_collection
            .find_one_and_update(doc! {"_id": oid}, update)
            .return_document(ReturnDocument::After)
            .await
            .map_err(MongoQueryError)?
        {
            let feedback = self.doc_to_feedback(&doc)?;
            let feedback_response = SingleFeedbackResponse {
                status: "success",
                data: FeedbackData { feedback },
            };
            Ok(feedback_response)
        } else {
            Err(NotFoundError(id.to_string()))
        }
    }

    pub async fn delete_feedback(&self, id: &str) -> Result<()> {
        let oid = ObjectId::from_str(id).map_err(|_| InvalidIDError(id.to_owned()))?;
        let filter = doc! {"_id": oid };

        let result = self
            .collection
            .delete_one(filter)
            .await
            .map_err(MongoQueryError)?;

        match result.deleted_count {
            0 => Err(NotFoundError(id.to_string())),
            _ => Ok(()),
        }
    }

    fn doc_to_feedback(&self, feedback: &FeedbackModel) -> Result<FeedbackResponse> {
        let feedback_response = FeedbackResponse {
            id: feedback.id.to_hex(),
            name: feedback.name.to_owned(),
            email: feedback.email.to_owned(),
            feedback: feedback.feedback.to_owned(),
            rating: feedback.rating.to_owned(),
            status: feedback.status.to_owned(),
            createdAt: feedback.createdAt,
            updatedAt: feedback.updatedAt,
        };

        Ok(feedback_response)
    }

    fn create_feedback_document(
        &self,
        body: &CreateFeedbackSchema,
        status: String,
    ) -> Result<bson::Document> {
        let serialized_data = bson::to_bson(body).map_err(MongoSerializeBsonError)?;
        let document = serialized_data.as_document().unwrap();

        let datetime = Utc::now();

        let mut doc_with_dates = doc! {
        "createdAt": datetime,
        "updatedAt": datetime,
        "status": status        };
        doc_with_dates.extend(document.clone());

        Ok(doc_with_dates)
    }
}
