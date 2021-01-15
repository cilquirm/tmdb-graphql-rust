use async_graphql::{
    Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject, Result, 
    http::{playground_source, GraphQLPlaygroundConfig}
};
use tide::{http::mime, Body, Response, StatusCode};
use serde::{Serialize, Deserialize};

mod rest_client {
  use surf::{Client, RequestBuilder, Url, middleware::Logger  };

  pub struct RestClient {
    base_url: String,
    client: Client,
    auth_token: String
  }

  impl RestClient {
    pub fn new_auth(base_url: String, auth_token: String) -> Self {
      let mut client = Client::new().with(Logger::new());
      client.set_base_url(Url::parse(base_url.as_str()).unwrap());

      RestClient {
        base_url,
        client,
        auth_token
      }
    }

    pub fn get(&self, path: String) -> RequestBuilder {
      (&self.client).get(&self.concat_url(path)).header("Authorization", &self.auth_token)
    }

    fn concat_url(&self, path: String) -> String {
      format!("{}{}", &self.base_url, path)
    }
  }
}

#[derive(SimpleObject)]
pub struct Demo {
  pub id: usize,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, SimpleObject)]
struct Genre {
  id: i32,
  name: String
}

#[derive(Serialize, Deserialize)]
struct Genres {
  genres: Vec<Genre>
}

struct Query;

#[Object]
impl Query {
  async fn demo(&self) -> Demo {
    Demo { id: 42 }
  }

  #[allow(non_snake_case)]
  async fn getMovieGenres(&self, ctx: &Context<'_> ) -> Result<Vec<Genre>> {
    let client = ctx.data::<rest_client::RestClient>()?;
    let genres: Genres = client.get("/genre/movie/list".to_string()).recv_json().await?;
    Ok(genres.genres)
  }

  #[allow(non_snake_case)]
  async fn getTVGenres(&self, ctx: &Context<'_> ) -> Result<Vec<Genre>> {
    let client = ctx.data::<rest_client::RestClient>()?;
    let genres: Genres = client.get("/genre/tv/list".to_string()).recv_json().await?;
    Ok(genres.genres)
  }

}

#[async_std::main]
async fn main() -> tide::Result<()> {
  let mut app = tide::new();

  let mut settings = config::Config::default();
  settings.merge(config::Environment::with_prefix("TMDB")).unwrap();

  let schema= Schema::build(
    Query, EmptyMutation, EmptySubscription
  ).data(
    rest_client::RestClient::new_auth(
      "https://api.themoviedb.org/3".to_string(), 
      settings.get_str("AUTH_TOKEN").unwrap()
    ) 
  ).finish();

  // add tide endpoint
  app.at("/graphql")
    .post(async_graphql_tide::endpoint(schema));

  // enable graphql playground
  app.at("/").get(|_| async move {
    Ok(Response::builder(StatusCode::Ok)
      .body(Body::from_string(playground_source(
      // note that the playground needs to know
      // the path to the graphql endpoint
GraphQLPlaygroundConfig::new("/graphql"),
    )))
    .content_type(mime::HTML)
    .build())
  });

  Ok(app.listen("127.0.0.1:8080").await?)
}