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
use tokio::net::TcpListener; // Import TcpListener

mod model;
mod state;

use state::AppState;
use model::{MissionsData, Mission}; // Suppress warning if not used, but we use Mission

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = AppState::new().expect("Failed to load missions");

    let app = Router::new()
        .route("/", get(index))
        .route("/partials/splash.html", get(splash))
        .route("/partials/coop.html", get(coop))
        .route("/partials/solo.html", get(solo))
        .route("/partials/all_missions.html", get(all_missions))
        .route("/partials/randomizer.html", get(randomizer))
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service("/style.css", ServeDir::new("style.css"))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
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
    // Manual rendering:
    let html = IndexTemplate { view }.render().unwrap();
    Html(html)
}

#[derive(Template)]
#[template(path = "partials/splash.html")]
struct SplashTemplate;

async fn splash() -> Html<String> {
    let html = SplashTemplate.render().unwrap();
    Html(html)
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
    nested_gacha_items: Option<Vec<String>>,
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

    let (nested_mission, nested_gacha_items) = if mission.needs_coop_singles {
         let sub_missions = &state.missions.coop_single;
         let sub = sub_missions.choose(&mut rng).cloned();
         
         let gacha = if let Some(ref m) = sub {
             if m.needs_gacha_item_checklist {
                 Some(model::GACHA_ITEMS.iter().map(|s| format!("assets/items/{}.png", s)).collect())
             } else {
                 None
             }
         } else {
             None
         };
         (sub, gacha)
    } else {
        (None, None)
    };

    let template = CoopTemplate {
        seed: seed_str,
        mission,
        nested_mission,
        random_item_img,
        random_loadout,
        random_number,
        nested_gacha_items,
    };

    let html = template.render().unwrap();
    Html(html)
}

#[derive(Template)]
#[template(path = "partials/solo.html")]
struct SoloTemplate {
    seed: String,
    mission: Mission,
    random_item_img: Option<String>,
    random_loadout: Option<Loadout>,
    random_number: Option<u32>,
}

async fn solo(
    State(state): State<AppState>,
    Query(params): Query<ViewParams>,
) -> Html<String> {
    let seed_str = params.seed.unwrap_or_else(|| {
        rand::thread_rng().gen::<u32>().to_string()
    });
    
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&seed_str, &mut hasher);
    let seed_u64 = std::hash::Hasher::finish(&hasher);
    let mut rng = StdRng::seed_from_u64(seed_u64);

    let missions = &state.missions.missions;
    let mission = missions.choose(&mut rng).unwrap().clone();
    
    // Resolve dynamic content (Same logic as coop, effectively)
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

    let template = SoloTemplate {
        seed: seed_str,
        mission,
        random_item_img,
        random_loadout,
        random_number,
    };

    let html = template.render().unwrap();
    Html(html)
}

#[derive(Template)]
#[template(path = "partials/all_missions.html")]
struct AllMissionsTemplate<'a> {
    missions: &'a MissionsData,
}

async fn all_missions(State(state): State<AppState>) -> Html<String> {
    let template = AllMissionsTemplate {
        missions: &state.missions,
    };
    let html = template.render().unwrap();
    Html(html)
}

#[derive(Template)]
#[template(path = "partials/randomizer.html")]
struct RandomizerTemplate {
    seed: String,
    loadout: Loadout,
}

async fn randomizer(Query(params): Query<ViewParams>) -> Html<String> {
    let seed_str = params.seed.unwrap_or_else(|| {
        rand::thread_rng().gen::<u32>().to_string()
    });
    
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&seed_str, &mut hasher);
    let seed_u64 = std::hash::Hasher::finish(&hasher);
    let mut rng = StdRng::seed_from_u64(seed_u64);

    let c = model::CHARACTERS.choose(&mut rng).unwrap();
    let k = model::KARTS.choose(&mut rng).unwrap();
    let w = model::WHEELS.choose(&mut rng).unwrap();
    let g = model::GLIDERS.choose(&mut rng).unwrap();
    
    let loadout = Loadout {
        char_img: format!("assets/characters/{}.webp", c.to_lowercase().replace(" ", "_")),
        char_name: c.to_string(),
        kart_img: format!("assets/karts/{}.webp", k.to_lowercase().replace(" ", "_")),
        kart_name: k.to_string(),
        wheel_img: format!("assets/wheels/{}.webp", w.to_lowercase().replace(" ", "_")),
        wheel_name: w.to_string(),
        glider_img: format!("assets/gliders/{}.webp", g.to_lowercase().replace(" ", "_")),
        glider_name: g.to_string(),
    };

    let template = RandomizerTemplate {
        seed: seed_str,
        loadout,
    };

    let html = template.render().unwrap();
    Html(html)
}
