use askama::Template;
use axum::{
    extract,
    handler::HandlerWithoutStateExt,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use std::{fs::File, io::Read, net::SocketAddr, path::Path};
use tower_http::services::ServeDir;

#[derive(Template)]
#[template(path = "index.html", escape = "none")]
struct PageTemplate {
    title: String,
    outlet: String,
}

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}

#[tokio::main]
async fn main() {
    let serve_dir = ServeDir::new("assets").not_found_service(handle_404.into_service());
    let app = Router::new()
        .route(
            "/",
            get(|| async {
                HtmlTemplate(PageTemplate {
                    title: "Home".to_owned(),
                    outlet: get_md(String::from("index")),
                })
            }),
        )
        .route("/*pages", get(resp))
        .nest_service("/static", serve_dir.clone())
        .fallback_service(serve_dir);

    let addr = SocketAddr::from(([127, 0, 0, 1], 5173));
    println!("Server is listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handle_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        HtmlTemplate(PageTemplate {
            title: "404".to_owned(),
            outlet: markdown::to_html("Not Found"),
        }),
    )
}

async fn resp(extract::Path(pages): extract::Path<String>) -> impl IntoResponse {
    let page = PageTemplate {
        title: "".to_owned(),
        outlet: get_md(pages),
    };

    HtmlTemplate(page)
}

fn get_md(pages: String) -> String {
    let content_path_str = format!("./contents/{}.md", pages.as_str());
    let path = Path::new(content_path_str.as_str());

    if path.exists() {
        match File::open(&path) {
            Ok(mut file) => {
                let mut content = String::new();
                file.read_to_string(&mut content).unwrap();
                markdown::to_html(content.as_str())
            }
            Err(_) => markdown::to_html("Not Found"),
        }
    } else {
        markdown::to_html("Not Found")
    }
}
