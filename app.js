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
async function init() {
  try {
    const response = await fetch('missions.json');
    if (!response.ok) throw new Error(`HTTP error! status: ${response.status}`);
    const data = await response.json();
    state.missions = data;

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
        } else if (evt.detail.pathInfo.requestPath.includes('randomizer.html')) {
          // Ensure seed is set if not present in URL for randomizer view
          if (!seed) {
            seed = generateRandomString();
            const newUrl = new URL(window.location.href);
            newUrl.searchParams.set('seed', seed);
            newUrl.searchParams.set('view', 'randomizer');
            window.history.replaceState(null, '', newUrl.toString());
          }
          state.seed = seed;
          renderRandomizer(document.getElementById('randomizer-container'));
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
    } else if (view === 'randomizer') {
      state.seed = seed || generateRandomString();
      htmx.ajax('GET', 'partials/randomizer.html', '#app-content').then(() => {
        renderRandomizer(document.getElementById('randomizer-container'));
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

function instantiateTemplate(id) {
  const template = document.getElementById(id);
  if (!template) {
    console.error(`Template ${id} not found`);
    return document.createElement('div');
  }
  return template.content.cloneNode(true);
}

function renderRandomizer(container) {
  if (!container) return;
  const layout = instantiateTemplate('randomizer-layout-template');

  // Set Seed
  layout.querySelector('.js-seed-display').textContent = `Seed: ${state.seed}`;

  // Set Reroll
  layout.querySelector('.js-reroll-btn').onclick = () => {
    window.location.href = `?view=randomizer&seed=${generateRandomString()}`;
  };

  const loadoutContainer = layout.querySelector('.js-loadout-container');
  loadoutContainer.appendChild(renderRandomLoadout(state.seed));

  container.innerHTML = '';
  container.appendChild(layout);
  htmx.process(container);
}

function renderSoloRandom(container) {
  if (!container) return;
  const rng = createRng(state.seed);
  const shuffledMissions = shuffle([...state.missions.missions], rng);
  const mission1 = shuffledMissions[0];
  const mission2 = shuffledMissions[1];

  const layout = instantiateTemplate('solo-layout-template');

  // Set Seed
  layout.querySelector('.js-seed-display').textContent = `Seed: ${state.seed}`;

  // Set Reroll
  layout.querySelector('.js-reroll-btn').onclick = () => {
    window.location.href = `?view=solo&seed=${generateRandomString()}`;
  };

  const missionContainer = layout.querySelector('.js-mission-container');
  missionContainer.appendChild(renderMissionCard(mission1, state.seed, 1));
  missionContainer.appendChild(renderMissionCard(mission2, state.seed, 2));

  container.innerHTML = '';
  container.appendChild(layout);
  htmx.process(container);
}

function renderCoopRandom(container) {
  if (!container) return;
  const rng = createRng(state.seed);
  const shuffledMissions = shuffle([...state.missions.coop_granprix], rng);
  const mission = shuffledMissions[0];

  const layout = instantiateTemplate('coop-layout-template');

  // Set Seed
  layout.querySelector('.js-seed-display').textContent = `Seed: ${state.seed}`;

  // Set Reroll
  layout.querySelector('.js-reroll-btn').onclick = () => {
    window.location.href = `?view=coop&seed=${generateRandomString()}`;
  };

  const missionContainer = layout.querySelector('.js-mission-container');
  missionContainer.appendChild(renderMissionCard(mission, state.seed, 1));

  container.innerHTML = '';
  container.appendChild(layout);
  htmx.process(container);
}

function renderMissionCard(mission, seed, index) {
  const cardFragment = instantiateTemplate('mission-card-template');
  const cardElement = cardFragment.querySelector('.card'); // We mostly need to return a node, but the fragment contains it.

  cardFragment.querySelector('.js-mission-name').textContent = mission.name;
  cardFragment.querySelector('.js-mission-desc').textContent = mission.description;

  // Details
  if (mission.details && mission.details.length > 0) {
    const detailsContainer = cardFragment.querySelector('.js-mission-details');
    const list = cardFragment.querySelector('.js-details-list');
    detailsContainer.classList.remove('hidden');
    mission.details.forEach(d => {
      const li = document.createElement('li');
      li.textContent = d;
      list.appendChild(li);
    });
  }

  const extraContent = cardFragment.querySelector('.js-extra-content');

  // Logic for sub-randoms
  if (mission.needs_random_item) {
    extraContent.appendChild(renderRandomItem(seed));
  }

  if (mission.needs_random_loadout) {
    extraContent.appendChild(renderRandomLoadout(seed));
  }

  if (mission.needs_random_number) {
    extraContent.appendChild(renderRandomNumber(seed, mission.needs_random_number));
  }

  if (mission.needs_coop_singles) {
    const nestedTemplate = instantiateTemplate('nested-mission-template');

    // Pick a single race mission
    const rng = createRng(seed);
    const singles = shuffle([...state.missions.coop_single], rng);
    const singleMission = singles[1];

    const nestedContainer = nestedTemplate.querySelector('.js-nested-mission-container');
    nestedContainer.appendChild(renderMissionCard(singleMission, seed + '_nested', 0));

    extraContent.appendChild(nestedTemplate);
  }

  // Use the fragment or the element? 
  // appendChild works with fragments.
  // But we want to return a Node that can be appended.
  return cardFragment;
}

function renderRandomItem(seed) {
  const rng = createRng(seed);
  const idx = Math.floor(rng() * ITEMS.length);
  const item = ITEMS[idx];

  const fragment = instantiateTemplate('random-item-template');
  const img = fragment.querySelector('.js-item-img');
  img.src = `assets/items/${item}.png`;
  img.alt = item;

  return fragment;
}

function renderRandomLoadout(seed) {
  const rng = createRng(seed);
  const getRand = (arr) => arr[Math.floor(rng() * arr.length)];
  const char = getRand(CHARACTERS);
  const kart = getRand(KARTS);
  const wheel = getRand(WHEELS);
  const glider = getRand(GLIDERS);

  const fragment = instantiateTemplate('random-loadout-template');

  const setPart = (role, name, folder) => {
    fragment.querySelector(`.js-${role}-img`).src = `assets/${folder}/${name.toLowerCase().replace(/ /g, '_')}.webp`;
    fragment.querySelector(`.js-${role}-name`).textContent = name;
  };

  setPart('char', char, 'characters');
  setPart('kart', kart, 'karts');
  setPart('wheel', wheel, 'wheels');
  setPart('glider', glider, 'gliders');

  return fragment;
}

function renderRandomNumber(seed, max) {
  const rng = createRng(seed);
  const num = Math.floor(rng() * max) + 1;

  const fragment = instantiateTemplate('random-number-template');
  fragment.querySelector('.js-number').textContent = num;

  return fragment;
}

// Start
init();
