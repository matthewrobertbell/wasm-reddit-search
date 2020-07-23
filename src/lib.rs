use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_sys::Document;

use gloo::events::EventListener;
use wasm_bindgen_futures::spawn_local;

#[derive(Deserialize, Debug)]
struct RedditResponse {
    pub data: RedditResponseData,
}

#[derive(Deserialize, Debug)]
pub struct RedditResponseData {
    pub children: Vec<RedditPostContainer>,
    pub after: String,
}

#[derive(Deserialize, Debug)]
pub struct RedditPostContainer {
    data: RedditPost,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RedditPost {
    id: String,
    url: String,
    num_comments: u64,
    ups: u64,
    downs: u64,
    permalink: String,
    subreddit_name_prefixed: String,
}

#[wasm_bindgen(start)]
pub async fn main() -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let button = document.get_element_by_id("search-button").unwrap();

    let on_click = EventListener::new(&button, "click", move |_event| {
        let document = document.clone();
        spawn_local(async {
            let _ = search(document).await;
        });
    });
    on_click.forget();

    Ok(())
}

async fn search(document: Document) -> Result<(), JsValue> {
    let handlebars = Handlebars::new();
    let template_string = r###"
        <a href="{{ url }}" target="_blank" class="d-block mb-4 h-100">
            <img class="img-fluid img-thumbnail" src="{{ url }}">
        </a>
        <a href="https://reddit.com{{ permalink }}" target="_blank">{{ subreddit_name_prefixed }} - {{ num_comments }} Comments, {{ ups }} Upvotes</a>
        "###;

    let search_input = document
        .get_element_by_id("search")
        .unwrap()
        .dyn_into::<web_sys::HtmlInputElement>()
        .unwrap();
    let search_term = search_input.value();
    let container = document.get_element_by_id("images-row").unwrap();
    container.set_inner_html("Loading...");
    let posts = get_reddit_posts(&search_term).await?;
    container.set_inner_html("");

    for post in posts {
        let rendered = handlebars
            .render_template(
                template_string,
                &serde_json::to_string(&post).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;

        let val = document.create_element("div")?;
        val.set_attribute("class", "col-lg-3 col-md-4 col-6")?;
        val.set_inner_html(&rendered);
        container.append_child(&val)?;
    }

    Ok(())
}

async fn get_reddit_posts(keyword: &str) -> Result<Vec<RedditPost>, JsValue> {
    let url = format!(
        "https://www.reddit.com/search.json?q={}&t=all&limit=100&sort=new",
        keyword
    );
    let r = reqwest::get(&url).await?;
    let response: RedditResponse = r
        .json()
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let image_extensions = ["png", "gif", "jpg", "jpeg"];
    Ok(response
        .data
        .children
        .into_iter()
        .map(|c| c.data)
        .filter(|p| p.ups > p.downs * 2)
        .filter(|p| image_extensions.iter().any(|ext| p.url.ends_with(ext)))
        .collect())
}
