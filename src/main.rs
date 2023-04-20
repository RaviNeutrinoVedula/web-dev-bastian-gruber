use warp::{Filter, reject::Reject, http::Method, filters::cors::CorsForbidden, Rejection, Reply, http::StatusCode};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
struct QuestionId(String);

impl std::fmt::Display for QuestionId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
	write!(f, "id: {}", self.0)
    }
}	

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

impl std::fmt::Display for Question {
    fn fmt (&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
	write!(f,
	       "{}, title: {}, content: {}, tags: {:?}",
	       self.id, self.title, self.content, self.tags
	)
    }
}

#[derive(Clone)]
struct Store {
    questions: HashMap<QuestionId, Question>,
}

impl Store {
    fn new() -> Self {
	Store {
            questions: Self::init(),
	}
    }

    fn init() -> HashMap<QuestionId, Question> {
	let file = include_str!("../questions.json");
	serde_json::from_str(file).expect("Can't read the question json file")
    }
}

#[derive(Debug)]
enum Error {
    ParseError(std::num::ParseIntError),
    MissingParameters,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
	match *self {
	    Error::ParseError(ref err) => {
		write!(f, "Cannot parse parameter: {}", err)
	    },
	    Error::MissingParameters => write!(f, "Missing parameter"),
	}
    }
}

impl Reject for Error {}

#[derive(Debug)]
struct Pagination {
    start: usize,
    end: usize,
}

fn extract_pagination (params: HashMap<String, String>)
		       -> Result<Pagination, Error> {
    if params.contains_key("start") && params.contains_key("end") {
	return Ok(Pagination {
	    start: params
		.get("start")
// the .get method returns an option because it can't be sure the key exists
// We can do the unsafe unwrap() here because we already checked for existence of both the parms in the if condn above.
		.unwrap()
		.parse::<usize>()
		.map_err(Error::ParseError)?,
	    end: params
		.get("end")
		.unwrap()
		.parse::<usize>()
		.map_err(Error::ParseError)?,
	});
    }
    Err(Error::MissingParameters)
}

async fn get_questions(
    params: HashMap<String, String>,
    store: Store) -> Result<impl warp::Reply, warp::Rejection> {
    // println!("{:?}", params);
    if !params.is_empty() {
	let pagination = extract_pagination(params)?;
	let res: Vec<Question> = store.questions
	    .values()
	    .cloned()
	    .collect();

// should be more robust: What if start > end, end is some very large num etc.?
	let res = &res[pagination.start..pagination.end];
	Ok(warp::reply::json(&res))
    } else {
	let res: Vec<Question> = store.questions
	    .values()
	    .cloned()
	    .collect();

	Ok(warp::reply::json(&res))
    }
}

#[tokio::main]
async fn main() {
    // let hello = warp::get()
    //     .map(|| format!("Hello, World!"));

    let store = Store::new();
    let store_filter = warp::any()
        .map(move || store.clone());
    
    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(
	    &[Method::PUT, Method::DELETE, Method::GET, Method::POST]
	);
    	
    let get_questions = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter)
        .and_then(get_questions)
        .recover(return_error);

    let routes = get_questions.with(cors);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}


async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(error) = r.find::<Error>() {
	Ok(warp::reply::with_status(
	    error.to_string(),
	    StatusCode::RANGE_NOT_SATISFIABLE,
	))
    } else if let Some(error) = r.find::<CorsForbidden>() {
	Ok(warp::reply::with_status(
	    error.to_string(),
	    StatusCode::FORBIDDEN,
	))
	} else {
	    Ok(warp::reply::with_status(
		"Route not found".to_string(),
		StatusCode::NOT_FOUND,
	    ))
	}
}
    
