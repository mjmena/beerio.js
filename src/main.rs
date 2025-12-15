use axum::{
    extract::{Query, State},
    response::Html,
    routing::get,
    Router,
};
use serde::Deserialize;
use tower_http::services::ServeDir;
use std::sync::Arc;
use askama::Template;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::Rng;

mod model;
mod state;

use state::AppState;
use model::{MissionsData, Mission};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = AppState::new().expect("Failed to load missions");

    let app = Router::new()
        .route("/", get(index))
        .route("/partials/splash.html", get(splash))
        .route("/partials/coop.html", get(coop))
        // .route("/partials/solo.html", get(solo)) // TODO
        // .route("/partials/randomizer.html", get(randomizer)) // TODO
        // .route("/partials/all_missions.html", get(all_missions)) // TODO
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service("/style.css", ServeDir::new("style.css")) // If we keep it flat
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct ViewParams {
    view: Option<String>,
    seed: Option<String>,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    view: String,
}

async fn index(Query(params): Query<ViewParams>) -> Html<String> {
    let view = params.view.unwrap_or_else(|| "splash".to_string());
    Html(IndexTemplate { view }.render().unwrap())
}

#[derive(Template)]
#[template(path = "partials/splash.html")]
struct SplashTemplate;

async fn splash() -> Html<String> {
    Html(SplashTemplate.render().unwrap())
}

#[derive(Template)]
#[template(path = "partials/coop.html")]
struct CoopTemplate {
    seed: String,
    mission: Mission,
    // We might need to pass nested data for complex missions
    nested_mission: Option<Mission>,
    random_item_img: Option<String>,
    random_loadout: Option<Loadout>,
    random_number: Option<u32>,
}

#[derive(Clone)] // Added Clone here
struct Loadout {
    char_img: String,
    char_name: String,
    kart_img: String,
    kart_name: String,
    wheel_img: String,
    wheel_name: String,
    glider_img: String,
    glider_name: String,
}

async fn coop(
    State(state): State<AppState>,
    Query(params): Query<ViewParams>,
) -> Html<String> {
    let seed_str = params.seed.unwrap_or_else(|| {
        rand::thread_rng().gen::<u32>().to_string()
    });
    
    // Create RNG from seed
    // Simple way: hash string to u64
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&seed_str, &mut hasher);
    let seed_u64 = std::hash::Hasher::finish(&hasher);
    let mut rng = StdRng::seed_from_u64(seed_u64);

    let missions = &state.missions.coop_granprix;
    let mission = missions.choose(&mut rng).unwrap().clone();
    
    // Resolve dynamic content
    let random_item_img = if mission.needs_random_item {
        let item = model::ITEMS.choose(&mut rng).unwrap();
        Some(format!("assets/items/{}.png", item))
    } else {
        None
    };

    let random_loadout = if mission.needs_random_loadout {
        let c = model::CHARACTERS.choose(&mut rng).unwrap();
        let k = model::KARTS.choose(&mut rng).unwrap();
        let w = model::WHEELS.choose(&mut rng).unwrap();
        let g = model::GLIDERS.choose(&mut rng).unwrap();
        
        Some(Loadout {
            char_img: format!("assets/characters/{}.webp", c.to_lowercase().replace(" ", "_")),
            char_name: c.to_string(),
            kart_img: format!("assets/karts/{}.webp", k.to_lowercase().replace(" ", "_")),
            kart_name: k.to_string(),
            wheel_img: format!("assets/wheels/{}.webp", w.to_lowercase().replace(" ", "_")),
            wheel_name: w.to_string(),
            glider_img: format!("assets/gliders/{}.webp", g.to_lowercase().replace(" ", "_")),
            glider_name: g.to_string(),
        })
    } else {
        None
    };

    let random_number = mission.needs_random_number.map(|max| {
         rng.gen_range(1..=max)
    });

    let nested_mission = if mission.needs_coop_singles {
         let sub_missions = &state.missions.coop_single;
         // We might want to ensure we don't pick the same one if we want that logic
         // But for now just random
         sub_missions.choose(&mut rng).cloned()
    } else {
        None
    };

    let template = CoopTemplate {
        seed: seed_str,
        mission,
        nested_mission,
        random_item_img,
        random_loadout,
        random_number,
    };

    Html(template.render().unwrap())
}
