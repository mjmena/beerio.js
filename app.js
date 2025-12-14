const state = {
  missions: null,
  seed: null,
  view: 'splash', // splash, solo, coop
  generatedData: {
    solo: [],
    coop: []
  }
};

// CONSTANTS
const ITEMS = [
  "coin", "banana", "triple_banana", "green_shell", "triple_green_shells",
  "red_shell", "triple_red_shells", "mushroom", "triple_mushrooms", "golden_mushroom",
  "super_star", "lightning", "bob-omb", "boo", "fire_flower", "boomerang_flower",
  "piranha_plant", "bullet_bill", "spiny_shell", "super_horn", "blooper", "crazy_eight"
];

const GACHA_ITEMS = [
  "bob-omb", "super_horn", "boomerang_flower", "fire_flower", "piranha_plant", "boo", "crazy_eight"
];

const KARTS = [
  "Standard Kart", "Pipe Frame", "B Dasher", "Mach 8", "Steel Driver", "Cat Cruiser",
  "Circuit Special", "Tri-Speeder", "Badwagon", "Prancer", "Biddybuggy", "Landship",
  "Sneeker", "Sports Coupe", "GLA", "W 25 Silver Arrow", "300 SL Roadster", "Blue Falcon",
  "Tanooki Kart", "Bone Rattler", "Inkstriker", "Master Cycle", "Streetle", "P-Wing",
  "Koopa Clown", "Standard Bike", "Comet", "Sport Bike", "The Duke", "Flame Rider",
  "Varmint", "Mr. Scooty", "Jet Bike", "Yoshi Bike", "Master Cycle Zero", "City Tripper"
];

const WHEELS = [
  "Standard", "Monster", "Roller", "Slim", "Slick", "Metal", "Button", "Off-Road",
  "Sponge", "Wood", "Cushion", "Blue Standard", "Hot Monster", "Azure Roller",
  "Crimson Slim", "Cyber Slick", "Retro Off-Road", "GLA Tires", "Triforce Tires",
  "Leaf Tires", "Ancient Tires"
];

const GLIDERS = [
  "Super Glider", "Cloud Glider", "Wario Wing", "Waddle Wing", "Peach Parasol",
  "Parachute", "Parafoil", "Flower Glider", "Bowser Kite", "Plane Glider",
  "MKTV Parafoil", "Hylian Kite", "Paper Glider", "Paraglider"
];

const CHARACTERS = [
  "Baby Daisy", "Baby Luigi", "Baby Mario", "Baby Peach", "Baby Rosalina", "Birdo",
  "Cat Peach", "Dry Bones", "Lemmy", "Bowser Jr.", "Daisy", "Diddy Kong", "Iggy",
  "Inkling Boy", "Inkling Girl", "Isabelle", "Kamek", "Koopa Troopa", "Lakitu",
  "Larry", "Link", "Luigi", "Ludwig", "Mario", "Morton", "Pauline", "Peach",
  "Peachette", "Rosalina", "Roy", "Shy Guy", "Toad", "Toadette", "Villager",
  "Wendy", "Wiggler", "Yoshi", "Bowser", "Donkey Kong", "Dry Bowser", "Funky Kong",
  "King Boo", "Metal Mario", "Petey Piranha", "Pink Gold Peach", "Wario", "Waluigi"
];

// Utils
function getHash() {
  return window.location.hash.slice(1) || 'splash';
}

function setHash(hash) {
  if (hash === 'solo' || hash === 'coop') {
    const url = new URL(window.location);
    if (!url.searchParams.has('seed')) {
      url.searchParams.set('seed', generateRandomString());
      window.history.replaceState(null, '', url.toString());
    }
  }
  window.location.hash = hash;
}

function getSeedFromUrl() {
  const url = new URL(window.location);
  return url.searchParams.get('seed') || generateRandomString();
}

function generateRandomString() {
  return Math.random().toString(36).substring(2, 7);
}

// PRNG - Mulberry32
// Simple seeded RNG that is sufficient for this purpose
function cyrb128(str) {
  let h1 = 1779033703, h2 = 3144134277,
    h3 = 1013904242, h4 = 2773480762;
  for (let i = 0, k; i < str.length; i++) {
    k = str.charCodeAt(i);
    h1 = h2 ^ Math.imul(h1 ^ k, 597399067);
    h2 = h3 ^ Math.imul(h2 ^ k, 2869860233);
    h3 = h4 ^ Math.imul(h3 ^ k, 951274213);
    h4 = h1 ^ Math.imul(h4 ^ k, 2716044179);
  }
  h1 = Math.imul(h3 ^ (h1 >>> 18), 597399067);
  h2 = Math.imul(h4 ^ (h2 >>> 22), 2869860233);
  h3 = Math.imul(h1 ^ (h3 >>> 17), 951274213);
  h4 = Math.imul(h2 ^ (h4 >>> 19), 2716044179);
  return [(h1 ^ h2 ^ h3 ^ h4) >>> 0, (h2 ^ h1) >>> 0, (h3 ^ h1) >>> 0, (h4 ^ h1) >>> 0];
}

function sfc32(a, b, c, d) {
  return function () {
    a >>>= 0; b >>>= 0; c >>>= 0; d >>>= 0;
    var t = (a + b) | 0;
    a = b ^ b >>> 9;
    b = c + (c << 3) | 0;
    c = (c << 21 | c >>> 11);
    d = (d + 1) | 0;
    t = (t + d) | 0;
    c = (c + t) | 0;
    return (t >>> 0) / 4294967296;
  }
}

function createRng(seedStr) {
  const seed = cyrb128(seedStr);
  return sfc32(seed[0], seed[1], seed[2], seed[3]);
}

function shuffle(array, rng) {
  let currentIndex = array.length, randomIndex;
  // While there remain elements to shuffle.
  while (currentIndex != 0) {
    // Pick a remaining element.
    randomIndex = Math.floor(rng() * currentIndex);
    currentIndex--;
    // And swap it with the current element.
    [array[currentIndex], array[randomIndex]] = [
      array[randomIndex], array[currentIndex]];
  }
  return array;
}

// Init
function init() {
  try {
    // Embed data directly for local file compatibility
    state.missions = {
      "missions": [
        {
          "name": "Last Minute",
          "description": "Drink only during the 3rd lap of each race."
        },
        {
          "name": "Pacing",
          "description": "*Must* have 5 gulps per race excluding last race."
        },
        {
          "name": "Smuggler",
          "description": "Finish *two* races holding or using a powerful item.",
          "details": ["Applicable items: Star, lightning, bullet bill, blue shell, golden mushroom, endless 8."]
        },
        {
          "name": "Janitor",
          "description": "Clean up 8 stationary non-coin items."
        },
        {
          "name": "Forgot my Keys",
          "description": "On *two* races, starting after the drink line, you must touch twelfth place."
        },
        {
          "name": "Showboating",
          "description": "Can only start drinking in 1st place.",
          "details": ["You can keep drinking even if you leave 1st place.", "If you pause your chug and you're no longer in first, stop drinking."]
        },
        {
          "name": "Randomizer",
          "description": "Use this random loadout:",
          "needs_random_loadout": true
        },
        {
          "name": "Sheep",
          "description": "Can only start a drink after someone else puts down their drink.",
          "details": ["Online Only: You must announce your mission at the start of the Gran Prix"]
        },
        {
          "name": "Item Masher",
          "description": "Hold every item for less than 2 seconds before using it."
        },
        {
          "name": "Locked Out",
          "description": "On *two* races, you can't use your first item."
        },
        {
          "name": "One in the Chamber",
          "description": "Can't use any bullet bills.",
          "details": ["Lose them through getting shocked or ghosted."]
        },
        {
          "name": "Stockpile",
          "description": "Must always have a second item on hold before you hold or use an item.",
          "details": ["You can not use your item before picking up a double."]
        },
        {
          "name": "Picky Drinker",
          "description": "You can only drink while holding this random item:",
          "needs_random_item": true
        },
        {
          "name": "TheOddOne",
          "description": "Start drinking in an odd numbered place, stop drinking in an odd numbered place.",
          "details": ["You're playing with fire if you are pushing on 11th."]
        },
        {
          "name": "Banker",
          "description": "Drink while only at 10 coins."
        },
        {
          "name": "Designated Driver",
          "description": "Can only drink while Lakitu is carrying you.",
          "details": ["You are permitted 1 addition gulp upon landing."]
        },
        {
          "name": "Tavern Brawler",
          "description": "This random race number is the only race you can drink your entire beer:",
          "needs_random_number": 4
        },
        {
          "name": "Masochist",
          "description": "Designate a player at the start. Get hit by that player twice.",
          "details": ["Running into their hazardous behind counts."]
        },
        {
          "name": "Gary Oak",
          "description": "The first player who pisses you off during the Gran Prix is your rival. You must hit them 3 times."
        },
        {
          "name": "Too Shocked To Drink",
          "description": "Can only have 1 drink session per race. If you’re shocked, you can’t drink. If you are drinking when shocked, put down your drink.",
          "details": [
            "If you start drinking on lap 1, you can only drink 4 gulps.",
            "If you start drinking on lap 2, you can only drink 7 gulps.",
            "If you start drinking on lap 3, you can drink as long as you aren’t shocked.",
            "On race 4 lap 3, you can finish your drink."
          ]
        },
        {
          "name": "Secret Santa",
          "description": "Bestow 3 positive effects to others with any rotating item around you.",
          "details": ["This is a hard to see visual effect - the item isn't consumed."]
        },
        {
          "name": "Pack Rat",
          "description": "If you get 6th or 7th place in the Grand Prix, you win alongside the next best player.",
          "details": [
            "You must announce your mission at the start of the Gran Prix",
            "Deep inside, you didn't win.",
            "Other players are permitted to call out Pack Rat as their secret mission in reaction.",
            "Their mission becomes Pack Rat instead of what it originally was."
          ]
        },
        {
          "name": "Bumper Cars",
          "description": "You must bump all other players 4 times. Each player must be bumped once.",
          "details": [
            "You must announce your mission at the start of the Gran Prix",
            "A bump is any non-agressive contact with your vehicle and another one"
          ]
        },
        {
          "name": "The Mark",
          "description": "Mark-drift over 2 coin collections.",
          "details": ["Mark-drift: Pick up all coins (at least 3) from a string of coins designed to be picked up with proper steering/drifting."]
        },
        {
          "name": "Watch Your Back",
          "description": "While in first, you must toggle rear view camera at least every 2 seconds."
        },
        {
          "name": "Double or Nothing",
          "description": "You are unable to drink for the rest of the race once you pick up a non-double item block."
        },
        {
          "name": "Drowning in Sorrow",
          "description": "You can only drink while submerged in water."
        }
      ],
      "coop_granprix": [
        {
          "name": "Mission Marathon",
          "description": "At the start of each race, attempt a single race mission.",
          "details": ["You lose if you fail *half or more* of these challenges."],
          "needs_coop_singles": true
        },
        {
          "name": "Mimicry",
          "description": "Two players must finish with the exact same score.",
          "details": ["For the first half of the races, attempt a single race mission.", "You lose if you fail *more than half* of the single race challenges"],
          "needs_coop_singles": true
        },
        {
          "name": "Share in the Glory",
          "description": "Each player must finish in first. Someone different must always place first.",
          "details": ["If races outnumber players, no player can have more than two firsts over another player."]
        },
        {
          "name": "The Perfect Run",
          "description": "One player must finish 1st in every race."
        },
        {
          "name": "Cascade",
          "description": "A drink session can only be started once each race. Only one person can drink at a time. When one person finishes, another can start.",
          "details": ["You can’t partake twice in the same session."]
        },
        {
          "name": "With Friends Like These",
          "description": "Every player must hit each other player once."
        },
        {
          "name": "Jack of All Trades",
          "description": "Every item must be used at least once.",
          "all_items": true
        },
        {
          "name": "Blue%",
          "description": "One player must dodge a blue shell.",
          "details": ["While in first, any item use that disrupts or avoids the blue shell counts."]
        },
        {
          "name": "Identity Crisis",
          "description": "Whenever a lightning bolt or blooper occurs, everyone must pass the controller to another player."
        },
        {
          "name": "Collectathon",
          "description": "At any point in the race, all players must have 10 coins at the same time."
        },
        {
          "name": "Killing Spree (GP Edition)",
          "description": "Computers must be hit a number of times equal to the number of players times 4."
        },
        {
          "name": "Soul-locked Drinking (GP Edition)",
          "description": "Everyone must drink at the same time."
        },
        {
          "name": "Get Down Mr. President (GP Edition)",
          "description": "Designate one player before the race starts. They can’t get hit more than 6 times *and* must finish 1st - 3rd by the end of the Gran Prix.",
          "details": ["Shocks don’t count - they are an aspect of nature."]
        },
        {
          "name": "Tag Team (GP Edition)",
          "description": "Break into pairs or triples. If you’re not leading, you must always have line of sight of another teammate.",
          "details": ["If you aren’t, the person leading must stop accelerating.", "At the start of each race, you can willingly keep or swap your partners."]
        },
        {
          "name": "Squad Goals",
          "description": "Every player must use the same random loadout.",
          "needs_random_loadout": true
        }
      ],
      "coop_single": [
        {
          "name": "Clean Sweep",
          "description": "All players must be in the top placements",
          "details": ["Four players must finish as 1, 2, 3, and 4."]
        },
        {
          "name": "Closing Time",
          "description": "Everyone must finish their beer."
        },
        {
          "name": "Shock%",
          "description": "One player must shock dodge."
        },
        {
          "name": "Initial D",
          "description": "Each player must get 1 draft."
        },
        {
          "name": "Apes Strong Together",
          "description": "Each player must get fed 1 banana.",
          "details": ["The banana must be trailing a racer and not lying on the ground."]
        },
        {
          "name": "Killing Spree",
          "description": "Computers must be hit a number of times equal to the number of players times 2."
        },
        {
          "name": "Bullying",
          "description": "Designate one CPU before the race starts. They must get hit a number of times equal to the number of players."
        },
        {
          "name": "Soul-locked Drinking",
          "description": "Everyone must drink at the same time."
        },
        {
          "name": "Get Down Mr. President",
          "description": "Designate one player before the race starts. They can’t get hit more than 3 times *and* must finish 1st - 3rd.",
          "details": ["Shocks don’t count - they are an aspect of nature."]
        },
        {
          "name": "Tag Team",
          "description": "Break into pairs or triples. If you’re not leading, you must always have line of sight of another teammate.",
          "details": ["If you aren’t, the person leading must stop accelerating."]
        },
        {
          "name": "Gacha Addict",
          "description": "Collect 4 of these 7 rare items:",
          "needs_gacha_item_checklist": true
        },
        {
          "name": "Hivemind",
          "description": "Call out when using an item, everyone else must press the item button."
        },
        {
          "name": "YOU - SHALL NOT - PASS",
          "description": "Designate one player before the race starts. After the beer line, at no point can no other player pass the designated player."
        },
        {
          "name": "Hand Holding",
          "description": "Everyone must finish within 2 seconds of each other.",
          "details": ["Do this by waiting before the finish line on the 3rd lap."]
        },
        {
          "name": "Waiting in Line",
          "description": "While 1 player is using Bullet Bill, the other players can’t drive."
        }
      ]
    };

    // Initialize HTMX handling
    if (typeof htmx !== 'undefined') {
      htmx.on('htmx:afterSwap', (evt) => {
        // Get updated URL or state
        const url = new URL(window.location.href);
        const view = url.searchParams.get('view');
        let seed = url.searchParams.get('seed');

        // Determine which partial was loaded and render dynamic content
        if (evt.detail.pathInfo.requestPath.includes('solo.html')) {
          // Ensure seed is set if not present in URL for solo view
          if (!seed) {
            seed = generateRandomString();
            const newUrl = new URL(window.location.href);
            newUrl.searchParams.set('seed', seed);
            newUrl.searchParams.set('view', 'solo'); // Ensure view is also set
            window.history.replaceState(null, '', newUrl.toString());
          }
          state.seed = seed;
          renderSoloRandom(document.getElementById('solo-container'));
        } else if (evt.detail.pathInfo.requestPath.includes('coop.html')) {
          // Ensure seed is set if not present in URL for coop view
          if (!seed) {
            seed = generateRandomString();
            const newUrl = new URL(window.location.href);
            newUrl.searchParams.set('seed', seed);
            newUrl.searchParams.set('view', 'coop'); // Ensure view is also set
            window.history.replaceState(null, '', newUrl.toString());
          }
          state.seed = seed;
          renderCoopRandom(document.getElementById('coop-container'));
        }
        // No action needed for splash.html as it's static
      });
    }

    // Initial Page Load Logic
    const url = new URL(window.location.href);
    const view = url.searchParams.get('view') || 'splash';
    const seed = url.searchParams.get('seed');

    const appContent = document.getElementById('app-content');

    if (view === 'solo') {
      state.seed = seed || generateRandomString();
      // Manually load partial for initial load if we are deep linking
      htmx.ajax('GET', 'partials/solo.html', '#app-content').then(() => {
        renderSoloRandom(document.getElementById('solo-container'));
      });
    } else if (view === 'coop') {
      state.seed = seed || generateRandomString();
      htmx.ajax('GET', 'partials/coop.html', '#app-content').then(() => {
        renderCoopRandom(document.getElementById('coop-container'));
      });
    } else {
      // Load Splash
      htmx.ajax('GET', 'partials/splash.html', '#app-content');
    }

  } catch (e) {
    console.error("Failed to load missions", e);
    document.getElementById('app').innerHTML = `<h1 class="text-center p-4">Error loading missions<br><small>${e.message}</small></h1>`;
  }
}

// The rest of the rendering functions (renderSoloRandom, renderCoopRandom, etc) remain valid
// but need to point to the correct container passed as argument.

function renderSoloRandom(container) {
  if (!container) return;
  const rng = createRng(state.seed);
  const shuffledMissions = shuffle([...state.missions.missions], rng);
  const mission1 = shuffledMissions[0];
  const mission2 = shuffledMissions[1];

  // Reroll is now full page reload or we can use HTMX to re-get the partial with new seed?
  // Easiest is location.href reload for now as per original code.
  const html = `
        <div class="flex flex-col h-full bg-slate-100 overflow-y-auto w-full">
            <div class="p-2 text-center bg-white shadow-sm z-10 sticky top-0 flex justify-between items-center">
                <button class="btn text-sm" hx-get="partials/splash.html" hx-target="#app-content" hx-push-url="?view=splash" hx-swap="innerHTML">Back</button>
                <div class="font-mono text-sm">Seed: ${state.seed}</div>
                <button class="btn text-sm" onclick="window.location.href='?view=solo&seed='+generateRandomString()">Reroll</button>
            </div>
            <div class="flex flex-col md:flex-row justify-evenly items-start p-4 gap-4">
                 ${renderMissionCard(mission1, state.seed, 1)}
                 ${renderMissionCard(mission2, state.seed, 2)}
            </div>
        </div>
    `;
  container.innerHTML = html;
  // Process HTMX on new content
  htmx.process(container);
}

function renderCoopRandom(container) {
  if (!container) return;
  const rng = createRng(state.seed);
  const shuffledMissions = shuffle([...state.missions.coop_granprix], rng);
  const mission = shuffledMissions[0];

  const html = `
        <div class="flex flex-col h-full bg-slate-100 overflow-y-auto w-full">
            <div class="p-2 text-center bg-white shadow-sm z-10 sticky top-0 flex justify-between items-center">
                <button class="btn text-sm" hx-get="partials/splash.html" hx-target="#app-content" hx-push-url="?view=splash" hx-swap="innerHTML">Back</button>
                <div class="font-mono text-sm">Seed: ${state.seed}</div>
                <button class="btn text-sm" onclick="window.location.href='?view=coop&seed='+generateRandomString()">Reroll</button>
            </div>
            <div class="flex flex-col items-center justify-center p-4">
                 ${renderMissionCard(mission, state.seed, 1)}
            </div>
        </div>
    `;
  container.innerHTML = html;
  // Process HTMX on new content
  htmx.process(container);
}

function renderMissionCard(mission, seed, index) {
  // We create a specific RNG for the sub-components based on seed + index to keep it stable but distinct
  // Actually rust code uses the SAME seed for random items.

  let extraHtml = '';

  // Logic for sub-randoms
  // Needs random item
  if (mission.needs_random_item) {
    extraHtml += renderRandomItem(seed);
  }

  // Needs random loadout
  if (mission.needs_random_loadout) {
    extraHtml += renderRandomLoadout(seed);
  }

  // Needs random number
  if (mission.needs_random_number) {
    extraHtml += renderRandomNumber(seed, mission.needs_random_number);
  }

  // Needs coop singles (nested mission)
  if (mission.needs_coop_singles) {
    // Pick a single race mission
    const rng = createRng(seed);
    // We match Rust: it creates new mission list and picks (min..max)
    // Rust uses `generate_numbers_from_hash` with original seed
    const singles = shuffle([...state.missions.coop_single], rng);
    const singleMission = singles[1]; // Rust uses index 1?
    extraHtml += `<div class="mt-4 border-t pt-4">
            <h4 class="font-bold text-red-600">Single Race Mission:</h4>
            ${renderMissionCard(singleMission, seed + '_nested', 0)}
        </div>`;
  }

  // Details list
  let detailsHtml = '';
  if (mission.details && mission.details.length > 0) {
    detailsHtml = `
            <div class="text-left mt-2">
                <h3 class="text-sm font-semibold text-gray-600">Details</h3>
                <ul class="list-disc pl-5 text-sm">
                    ${mission.details.map(d => `<li>${d}</li>`).join('')}
                </ul>
            </div>
        `;
  }

  return `
        <div class="card flex flex-col items-center">
            <div class="text-xl font-bold mb-2 text-center">${mission.name}</div>
            <div class="text-md mb-2 text-center">${mission.description}</div>
            ${detailsHtml}
            ${extraHtml}
        </div>
    `;
}

function renderRandomItem(seed) {
  const rng = createRng(seed);
  const idx = Math.floor(rng() * ITEMS.length);
  const item = ITEMS[idx];
  return `<div class="p-2"><img src="assets/items/${item}.png" class="img-contain w-32 h-32" alt="${item}"></div>`;
}

function renderRandomLoadout(seed) {
  const rng = createRng(seed);

  const getRand = (arr) => arr[Math.floor(rng() * arr.length)];

  const char = getRand(CHARACTERS);
  const kart = getRand(KARTS);
  const wheel = getRand(WHEELS);
  const glider = getRand(GLIDERS);

  const imgStyle = "w-24 h-24 object-scale-down";

  return `
        <div class="flex flex-wrap justify-center gap-2 mt-2">
            <div class="flex flex-col items-center">
                <img src="assets/characters/${char.toLowerCase().replace(/ /g, '_')}.webp" class="${imgStyle}">
                <span class="text-xs">${char}</span>
            </div>
            <div class="flex flex-col items-center">
                <img src="assets/karts/${kart.toLowerCase().replace(/ /g, '_')}.webp" class="${imgStyle}">
                <span class="text-xs">${kart}</span>
            </div>
            <div class="flex flex-col items-center">
                <img src="assets/wheels/${wheel.toLowerCase().replace(/ /g, '_')}.webp" class="${imgStyle}">
                <span class="text-xs">${wheel}</span>
            </div>
            <div class="flex flex-col items-center">
                <img src="assets/gliders/${glider.toLowerCase().replace(/ /g, '_')}.webp" class="${imgStyle}">
                <span class="text-xs">${glider}</span>
            </div>
        </div>
    `;
}

function renderRandomNumber(seed, max) {
  const rng = createRng(seed);
  const num = Math.floor(rng() * max) + 1;
  return `<div class="text-4xl font-bold p-4">${num}</div>`;
}

// Start
init();
