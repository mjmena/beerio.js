use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mission {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub details: Vec<String>,
    #[serde(default)]
    pub needs_random_loadout: bool,
    #[serde(default)]
    pub needs_random_item: bool,
    #[serde(default)]
    pub needs_random_number: Option<u32>,
    #[serde(default)]
    pub needs_coop_singles: bool,
    #[serde(default)]
    pub all_items: bool,
    #[serde(default)]
    pub needs_gacha_item_checklist: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionsData {
    pub missions: Vec<Mission>,
    pub coop_granprix: Vec<Mission>,
    pub coop_single: Vec<Mission>,
}

// Constants for Random Generation
pub const ITEMS: &[&str] = &[
    "coin", "banana", "triple_banana", "green_shell", "triple_green_shells",
    "red_shell", "triple_red_shells", "mushroom", "triple_mushrooms", "golden_mushroom",
    "super_star", "lightning", "bob-omb", "boo", "fire_flower", "boomerang_flower",
    "piranha_plant", "bullet_bill", "spiny_shell", "super_horn", "blooper", "crazy_eight",
];

pub const GACHA_ITEMS: &[&str] = &[
    "bob-omb", "super_horn", "boomerang_flower", "fire_flower", "piranha_plant", "boo", "crazy_eight",
];

pub const KARTS: &[&str] = &[
    "Standard Kart", "Pipe Frame", "B Dasher", "Mach 8", "Steel Driver", "Cat Cruiser",
    "Circuit Special", "Tri-Speeder", "Badwagon", "Prancer", "Biddybuggy", "Landship",
    "Sneeker", "Sports Coupe", "GLA", "W 25 Silver Arrow", "300 SL Roadster", "Blue Falcon",
    "Tanooki Kart", "Bone Rattler", "Inkstriker", "Master Cycle", "Streetle", "P-Wing",
    "Koopa Clown", "Standard Bike", "Comet", "Sport Bike", "The Duke", "Flame Rider",
    "Varmint", "Mr. Scooty", "Jet Bike", "Yoshi Bike", "Master Cycle Zero", "City Tripper",
];

pub const WHEELS: &[&str] = &[
    "Standard", "Monster", "Roller", "Slim", "Slick", "Metal", "Button", "Off-Road",
    "Sponge", "Wood", "Cushion", "Blue Standard", "Hot Monster", "Azure Roller",
    "Crimson Slim", "Cyber Slick", "Retro Off-Road", "GLA Tires", "Triforce Tires",
    "Leaf Tires", "Ancient Tires",
];

pub const GLIDERS: &[&str] = &[
    "Super Glider", "Cloud Glider", "Wario Wing", "Waddle Wing", "Peach Parasol",
    "Parachute", "Parafoil", "Flower Glider", "Bowser Kite", "Plane Glider",
    "MKTV Parafoil", "Hylian Kite", "Paper Glider", "Paraglider",
];

pub const CHARACTERS: &[&str] = &[
    "Baby Daisy", "Baby Luigi", "Baby Mario", "Baby Peach", "Baby Rosalina", "Birdo",
    "Cat Peach", "Dry Bones", "Lemmy", "Bowser Jr.", "Daisy", "Diddy Kong", "Iggy",
    "Inkling Boy", "Inkling Girl", "Isabelle", "Kamek", "Koopa Troopa", "Lakitu",
    "Larry", "Link", "Luigi", "Ludwig", "Mario", "Morton", "Pauline", "Peach",
    "Peachette", "Rosalina", "Roy", "Shy Guy", "Toad", "Toadette", "Villager",
    "Wendy", "Wiggler", "Yoshi", "Bowser", "Donkey Kong", "Dry Bowser", "Funky Kong",
    "King Boo", "Metal Mario", "Petey Piranha", "Pink Gold Peach", "Wario", "Waluigi",
];
