use std::io::{Error, ErrorKind};
use std::str::FromStr;
use warp::Filter;

#[derive(Debug)]
struct QuestionId(String);

impl FromStr for QuestionId {
    type Err = std::io::Error;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
	match id.is_empty() {
	    false => Ok(QuestionId(id.to_string())),
	    true => Err(Error::new(ErrorKind::InvalidInput, "No id provided")),
	}
    }
}

impl std::fmt::Display for QuestionId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
	write!(f, "id: {}", self.0)
    }
}	

#[derive(Debug)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

impl Question {
    fn new (id: QuestionId, title: String, content: String, tags: Option<Vec<String>>)
	-> Self {
	Question { id, title, content, tags }
    }
}

impl std::fmt::Display for Question {
    fn fmt (&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
	write!(f,
	       "{}, title: {}, content: {}, tags: {:?}",
	       self.id, self.title, self.content, self.tags
	)
    }
}

async fn get_questions() -> Result<impl warp::Reply, warp::Rejection> {
    let question = Question::new(
	QuestionId::from_str("1").expect("No id provided"),
	"First question".to_string(),
	"Content of first question".to_string(),
	Some(vec!("faq".to_string())),
    );

    Ok(warp::reply::json(
	&question
    ))
}

#[tokio::main]
async fn main() {
    // let hello = warp::get()
    //     .map(|| format!("Hello, World!"));

    let get_items = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and_then(get_questions);

    let routes = get_items;

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
