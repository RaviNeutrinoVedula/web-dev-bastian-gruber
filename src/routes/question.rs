use std::collections::HashMap;
use warp::http::StatusCode;

use crate::store::Store;
use crate::types::pagination::{extract_pagination, Pagination};
use crate::types::question::{Question, NewQuestion};
use crate::profanity::check_profanity;

use tracing::{event, instrument, Level};

#[instrument]
pub async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    event!(target: "practical_rust_book", Level::INFO, "querying questions");
    let mut pagination = Pagination::default();
    
    if !params.is_empty() {
	event!(Level::INFO, pagination = true);
        pagination = extract_pagination(params)?;
    }

    match store.get_questions(pagination.limit, pagination.offset)
	.await {
	    Ok(res) => Ok(warp::reply::json(&res)),
	    Err(e) => Err(warp::reject::custom(e)),
	}
}

pub async fn add_question(
    store: Store,
    new_question: NewQuestion,
) -> Result<impl warp::Reply, warp::Rejection> {

    let title = match check_profanity(new_question.title)
	.await {
	    Ok(res) => res,
	    Err(e) => return Err(warp::reject::custom(e)),
	};

    let content = match check_profanity(new_question.content)
	.await {
	    Ok(res) => res,
	    Err(e) => return Err(warp::reject::custom(e)),
	};    
    
    let question = NewQuestion {
	title,
	content,
	tags: new_question.tags,
    };

    match store.add_question(question)
        .await {
	    Ok(question) => Ok(warp::reply::json(&question)),
	    Err(e) => Err(warp::reject::custom(e)),
	}
}


pub async fn update_question(
    id: i32,
    store: Store,
    question: Question,
) -> Result<impl warp::Reply, warp::Rejection> {

    match store.update_question(question, id)
        .await {
	    Ok(res) => Ok(warp::reply::json(&res)),
	    Err(e) => Err(warp::reject::custom(e)),
	}
}

pub async fn delete_question(id: i32, store: Store)
			     -> Result<impl warp::Reply, warp::Rejection> {

    if let Err(e) = store.delete_question(id)
        .await {
	    return Err(warp::reject::custom(e));
	}

    Ok(warp::reply::with_status(
	format!("Question {} deleted", id),
	StatusCode::OK)
    )
}
