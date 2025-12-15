use axum::{
    extract::{Query, State},
    response::Html,
    routing::get,
    http::HeaderMap,
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
use percent_encoding::percent_decode_str;

mod model;
mod state;

use state::AppState;
use model::{MissionsData, Mission};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = AppState::new().expect("Failed to load missions");

    let app = Router::new()
        .route("/", get(root))
        .route("/{seed}", get(splash))
        .route("/{seed}/coop", get(coop))
        .route("/{seed}/solo", get(solo))
        .route("/{seed}/randomizer", get(randomizer))
        .route("/{seed}/all_missions", get(all_missions))
        .route("/{seed}/mission/{name}", get(mission_view))
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

async fn root() -> axum::response::Redirect {
    let seed = rand::thread_rng().gen::<u32>().to_string();
    axum::response::Redirect::to(&format!("/{}", seed))
}

#[derive(Template)]
#[template(path = "layout_wrapper.html")]
struct LayoutWrapperTemplate {
    content: String,
}

fn render_response(headers: HeaderMap, content: String) -> Html<String> {
    if headers.contains_key("hx-request") {
        Html(content)
    } else {
        let wrapper = LayoutWrapperTemplate { content };
        Html(wrapper.render().unwrap())
    }
}

#[derive(Template)]
#[template(path = "partials/splash.html")]
struct SplashTemplate {
    seed: String,
}

async fn splash(
    axum::extract::Path(seed): axum::extract::Path<String>,
    headers: HeaderMap,
) -> Html<String> {
    let template = SplashTemplate { seed };
    render_response(headers, template.render().unwrap())
}

#[derive(Template)]
#[template(path = "partials/coop.html")]
struct CoopTemplate {
    seed: String,
    next_seed: String,
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
    axum::extract::Path(seed): axum::extract::Path<String>,
    headers: HeaderMap,
) -> Html<String> {
    
    // Create RNG from seed
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&seed, &mut hasher);
    let seed_u64 = std::hash::Hasher::finish(&hasher);
    let mut rng = StdRng::seed_from_u64(seed_u64);
    let next_seed = rand::thread_rng().gen::<u32>().to_string();

    let missions = &state.missions.coop_granprix;
    let mission = missions.choose(&mut rng).unwrap().clone();
    
    // Resolve dynamic content
    let random_item_img = if mission.needs_random_item {
        let item = model::ITEMS.choose(&mut rng).unwrap();
        Some(format!("/assets/items/{}.png", item))
    } else {
        None
    };

    let random_loadout = if mission.needs_random_loadout {
        let c = model::CHARACTERS.choose(&mut rng).unwrap();
        let k = model::KARTS.choose(&mut rng).unwrap();
        let w = model::WHEELS.choose(&mut rng).unwrap();
        let g = model::GLIDERS.choose(&mut rng).unwrap();
        
        Some(Loadout {
            char_img: format!("/assets/characters/{}.webp", c.to_lowercase().replace(" ", "_")),
            char_name: c.to_string(),
            kart_img: format!("/assets/karts/{}.webp", k.to_lowercase().replace(" ", "_")),
            kart_name: k.to_string(),
            wheel_img: format!("/assets/wheels/{}.webp", w.to_lowercase().replace(" ", "_")),
            wheel_name: w.to_string(),
            glider_img: format!("/assets/gliders/{}.webp", g.to_lowercase().replace(" ", "_")),
            glider_name: g.to_string(),
        })
    } else {
        None
    };

    let random_number = mission.needs_random_number.map(|max| {
         rng.gen_range(1..=max)
    });

    let (nested_mission, nested_gacha_items) = if mission.needs_coop_singles {
         let sub = state.missions.coop_single.choose(&mut rng).cloned();
         let gacha = sub.as_ref().and_then(|m| {
             if m.needs_gacha_item_checklist {
                 Some(model::GACHA_ITEMS.iter().map(|s| format!("/assets/items/{}.png", s)).collect())
             } else { None }
         });
         (sub, gacha)
    } else { (None, None) };

    let template = CoopTemplate {
        seed,
        next_seed,
        mission,
        nested_mission,
        random_item_img,
        random_loadout,
        random_number,
        nested_gacha_items,
    };

    render_response(headers, template.render().unwrap())
}

#[derive(Template)]
#[template(path = "partials/solo.html")]
struct SoloTemplate {
    seed: String,
    next_seed: String,
    mission: Mission,
    nested_mission: Option<Mission>,
    random_item_img: Option<String>,
    random_loadout: Option<Loadout>,
    random_number: Option<u32>,
}

async fn solo(
    State(state): State<AppState>,
    axum::extract::Path(seed): axum::extract::Path<String>,
    headers: HeaderMap,
) -> Html<String> {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&seed, &mut hasher);
    let seed_u64 = std::hash::Hasher::finish(&hasher);
    let mut rng = StdRng::seed_from_u64(seed_u64);
    let next_seed = rand::thread_rng().gen::<u32>().to_string();
    
    let mission = state.missions.missions.choose(&mut rng).unwrap().clone();
    
    let random_item_img = if mission.needs_random_item {
        let item = model::ITEMS.choose(&mut rng).unwrap();
        Some(format!("/assets/items/{}.png", item))
    } else { None };

    let random_loadout = if mission.needs_random_loadout {
        let c = model::CHARACTERS.choose(&mut rng).unwrap();
        let k = model::KARTS.choose(&mut rng).unwrap();
        let w = model::WHEELS.choose(&mut rng).unwrap();
        let g = model::GLIDERS.choose(&mut rng).unwrap();
        
        Some(Loadout {
            char_img: format!("/assets/characters/{}.webp", c.to_lowercase().replace(" ", "_")),
            char_name: c.to_string(),
            kart_img: format!("/assets/karts/{}.webp", k.to_lowercase().replace(" ", "_")),
            kart_name: k.to_string(),
            wheel_img: format!("/assets/wheels/{}.webp", w.to_lowercase().replace(" ", "_")),
            wheel_name: w.to_string(),
            glider_img: format!("/assets/gliders/{}.webp", g.to_lowercase().replace(" ", "_")),
            glider_name: g.to_string(),
        })
    } else { None };

    let random_number = mission.needs_random_number.map(|max| rng.gen_range(1..=max));

    let nested_mission = if mission.needs_coop_singles {
         state.missions.coop_single.choose(&mut rng).cloned()
    } else { None };

    let template = SoloTemplate {
        seed,
        next_seed,
        mission,
        nested_mission,
        random_item_img,
        random_loadout,
        random_number,
    };
    render_response(headers, template.render().unwrap())
}

#[derive(Template)]
#[template(path = "partials/randomizer.html")]
struct RandomizerTemplate {
    seed: String,
    next_seed: String,
    loadout: Loadout,
}

async fn randomizer(
    axum::extract::Path(seed): axum::extract::Path<String>,
    headers: HeaderMap,
) -> Html<String> {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&seed, &mut hasher);
    let seed_u64 = std::hash::Hasher::finish(&hasher);
    let mut rng = StdRng::seed_from_u64(seed_u64);
    let next_seed = rand::thread_rng().gen::<u32>().to_string();
    
    let c = model::CHARACTERS.choose(&mut rng).unwrap();
    let k = model::KARTS.choose(&mut rng).unwrap();
    let w = model::WHEELS.choose(&mut rng).unwrap();
    let g = model::GLIDERS.choose(&mut rng).unwrap();
    
    let loadout = Loadout {
        char_img: format!("/assets/characters/{}.webp", c.to_lowercase().replace(" ", "_")),
        char_name: c.to_string(),
        kart_img: format!("/assets/karts/{}.webp", k.to_lowercase().replace(" ", "_")),
        kart_name: k.to_string(),
        wheel_img: format!("/assets/wheels/{}.webp", w.to_lowercase().replace(" ", "_")),
        wheel_name: w.to_string(),
        glider_img: format!("/assets/gliders/{}.webp", g.to_lowercase().replace(" ", "_")),
        glider_name: g.to_string(),
    };

    let template = RandomizerTemplate {
        seed,
        next_seed,
        loadout,
    };
    render_response(headers, template.render().unwrap())
}

#[derive(Template)]
#[template(path = "partials/all_missions.html")]
struct AllMissionsTemplate<'a> {
    missions: &'a MissionsData,
    seed: String,
}

async fn all_missions(
    State(state): State<AppState>,
    axum::extract::Path(seed): axum::extract::Path<String>,
    headers: HeaderMap,
) -> Html<String> {
    let template = AllMissionsTemplate {
        missions: &state.missions,
        seed,
    };
    render_response(headers, template.render().unwrap())
}

async fn mission_view(
    State(state): State<AppState>,
    axum::extract::Path((seed, name)): axum::extract::Path<(String, String)>,
    headers: HeaderMap,
) -> Html<String> {
    // Check main.rs imports for percent_encoding, if not present we might need to rely on direct match or add it
    // Assuming simple name match for now or re-adding helper function
    
    let mission = find_mission(&state.missions, &name);

    if let Some(mission) = mission {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(&seed, &mut hasher);
        let seed_u64 = std::hash::Hasher::finish(&hasher);
        let mut rng = StdRng::seed_from_u64(seed_u64);
        let next_seed = rand::thread_rng().gen::<u32>().to_string();

        let random_item_img = if mission.needs_random_item {
            let item = model::ITEMS.choose(&mut rng).unwrap();
            Some(format!("/assets/items/{}.png", item))
        } else { None };

        let random_loadout = if mission.needs_random_loadout {
            let c = model::CHARACTERS.choose(&mut rng).unwrap();
            let k = model::KARTS.choose(&mut rng).unwrap();
            let w = model::WHEELS.choose(&mut rng).unwrap();
            let g = model::GLIDERS.choose(&mut rng).unwrap();
            
            Some(Loadout {
                char_img: format!("/assets/characters/{}.webp", c.to_lowercase().replace(" ", "_")),
                char_name: c.to_string(),
                kart_img: format!("/assets/karts/{}.webp", k.to_lowercase().replace(" ", "_")),
                kart_name: k.to_string(),
                wheel_img: format!("/assets/wheels/{}.webp", w.to_lowercase().replace(" ", "_")),
                wheel_name: w.to_string(),
                glider_img: format!("/assets/gliders/{}.webp", g.to_lowercase().replace(" ", "_")),
                glider_name: g.to_string(),
            })
        } else { None };

        let random_number = mission.needs_random_number.map(|max| rng.gen_range(1..=max));
        
        // Populate nested_mission for SoloTemplate just in case
        let nested_mission = if mission.needs_coop_singles {
             let sub_missions = &state.missions.coop_single;
             sub_missions.choose(&mut rng).cloned()
        } else { None };

        let template = SoloTemplate {
            seed,
            next_seed,
            mission,
            nested_mission,
            random_item_img,
            random_loadout,
            random_number,
        };
        render_response(headers, template.render().unwrap())

    } else {
        Html("<h1>Mission Not Found</h1>".to_string())
    }
}

fn find_mission(data: &MissionsData, name: &str) -> Option<Mission> {
    // Simple lookup if encoding crate missing, otherwise use decode
    // We didn't re-add percent_encoding to imports in my recent 'Revert' step, 
    // but the previous Agent step 703 tried to add it. 
    // The USER removed it in 728.
    // I will try to use it assuming I can add the use statement or just do direct comparison if name is simple
    // Safer to decode.
    let name_decoded = percent_encoding::percent_decode_str(name).decode_utf8_lossy();
    
    data.missions.iter()
        .chain(data.coop_granprix.iter())
        .chain(data.coop_single.iter())
        .find(|m| m.name.eq_ignore_ascii_case(&name_decoded))
        .cloned()
}
