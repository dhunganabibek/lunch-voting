// use declarations pull structs, functions, and traits into the current namespace from other crates and libraries
// https://doc.rust-lang.org/reference/items/use-declarations.html
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Row, SqlitePool};

// #[] is a macro, and in this case declares an attribute, which applies metadata to the module, crate, or in this case, item below.
// https://doc.rust-lang.org/rust-by-example/attribute.html
// the derive attribute will try to automatically generate the implementations of the trait passed to it for the item below it
// https://doc.rust-lang.org/rust-by-example/trait/derive.html?highlight=derive#derive
// in this case, Clone, which allows us to use .clone() to create full copies of structures rather than just passing ownership
// https://doc.rust-lang.org/rust-by-example/trait/clone.html?highlight=clone#clone
#[derive(Clone)]
struct AppState {
    // Here we are defining a struct to carry shared state for the whole app
    db: SqlitePool, // This will hold the current SQLite database connection so all functions with the state can access it
}

// This macro makes the code run on the tokio runtime
// https://docs.rs/tokio/latest/tokio/
#[tokio::main]
async fn main() {
    // Initializes tracing subscriber, which allows for better diagnostics in asynchronous tokio operations
    // https://docs.rs/tracing-subscriber/latest/tracing_subscriber/index.html
    tracing_subscriber::fmt::init();
    // Initializes the database connection; in this case, just creates one in non-persistent memory
    // https://docs.rs/sqlx/latest/sqlx/type.SqlitePool.html
    let db = SqlitePool::connect("sqlite::memory:").await.unwrap();
    let _res = sqlx::query("CREATE TABLE IF NOT EXISTS votes 
        (id INTEGER PRIMARY KEY
        voter_name VARCHAR(255) NOT NULL, 
        restaurant_name VARCHAR(255) NOT NULL,
        ),")
        .execute(&db)
        .await
        .expect("Failed to create votes table");

    // Creates an app state instance with db set to the connection we just created
    let state = AppState { db };
    // Instantiates the server app, defines handlers, services, and state
    // https://docs.rs/axum/latest/axum/struct.Router.html
    // In this case, we are routing any requests to the /vote endpoint to the vote function as its handler
    // and adding the app state variable created above
    let app = Router::new().route("/vote", post(vote)).with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    // Run server and pause here, handling any incoming requests
    axum::serve(listener, app).await.unwrap();
}

// the Serialize trait from the serde crate allows the structure to be serialized into JSON
// https://docs.rs/serde/latest/serde/trait.Serialize.html
#[derive(Serialize)]
struct LunchVoting {
    votes: Vec<Restaurant>, // For this struct member, we are declaring it as a Vector who's elements are the Restaurant struct defined below
}

#[derive(Serialize)]
struct Restaurant {
    name: String,
    voters: Vec<String>,
}

// The Debug trait allows us to print instances of the struct without needing to implement special formatting functionality
// https://doc.rust-lang.org/rust-by-example/hello/print/print_debug.html?highlight=debug#debug
// The Deserialize trait from serde allows us to convert JSON into instances of this struct
// https://docs.rs/serde/latest/serde/trait.Deserialize.html
#[derive(Debug, Deserialize)]
struct VoteRequest {
    voter_name: String,
    restaurant_name: String,
}

// Here is where we define our /vote endpoint handler. It gets passed the app state, and the request JSON payload since it is a post
// request handler
// The parameters for axum handler functions are called extractors, since they pull in different parts of the request and app
// based on their type
// https://docs.rs/axum/latest/axum/handler/index.html
async fn vote(state: State<AppState>, req: Json<VoteRequest>) {
    dbg!(&req);
    let vote_req: VoteRequest = req.0;
    let res = save_vote(state, vote_req).await;
    dbg!(res);
}

// Here we are creating an enumeration. Enumerations are very flexible and powerful in Rust.
// For example, Rust's Result and Option types are just enumerations
// https://doc.rust-lang.org/rust-by-example/custom_types/enum.html?highlight=enum#enums
enum SaveVoteError {
    DbError(sqlx::Error),
    UnknownRestaurant(String),
}

// Here we declare a function to handle saving submitted votes to the database we created
async fn save_vote(state: State<AppState>, vote: VoteRequest) -> Result<(), sqlx::Error> {
    // We run the query with fetch_all to indicate that the entire result should be brought into a vector rather than streamed
    // https://docs.rs/sqlx/latest/sqlx/query/struct.Query.html
    let res = sqlx::query("SELECT * FROM votes")
                .fetch_all(&state.db)
                .await?;

    for r in res {
        println!("{} vote for {}", r.get::<&str,_>("voter_name"), r.get::<&str,_>("restaurant_name"));
    }

    Ok(())
}
