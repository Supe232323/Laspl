use crate::crypto::{self, EncryptedVault, MasterKey};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::State;
use uuid::Uuid;
use zeroize::Zeroize;

#[derive(Serialize, Deserialize, Clone)]
pub struct PasswordEntry {
    pub id: String,
    pub title: String,
    pub username: String,
    pub password: String,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub favorite: bool,
    pub category: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Serialize, Deserialize, Default)]
struct VaultData {
    entries: Vec<PasswordEntry>,
}

pub struct VaultState {
    key: Option<MasterKey>,
    data: VaultData,
    path: PathBuf,
}

impl Default for VaultState {
    fn default() -> Self {
        let path = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("laspl")
            .join("vault.laspl");
        Self {
            key: None,
            data: VaultData::default(),
            path,
        }
    }
}

pub struct AppState(pub Mutex<VaultState>);

fn ensure_parent(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn unlock_vault(
    password: String,
    state: State<'_, AppState>,
) -> Result<Vec<PasswordEntry>, String> {
    let mut st = state.0.lock().map_err(|e| e.to_string())?;

    if !st.path.exists() {
        let salt = crypto::generate_salt();
        let key = crypto::derive_key(&password, &salt).map_err(|e| e.to_string())?;
        let data = VaultData::default();
        let plaintext = serde_json::to_vec(&data).map_err(|e| e.to_string())?;
        let encrypted = crypto::encrypt(&key, &plaintext).map_err(|e| e.to_string())?;

        let vault = EncryptedVault {
            version: 1,
            salt: B64.encode(salt),
            data: encrypted,
        };
        ensure_parent(&st.path)?;
        let json = serde_json::to_string_pretty(&vault).map_err(|e| e.to_string())?;
        fs::write(&st.path, json).map_err(|e| e.to_string())?;

        st.key = Some(key);
        st.data = data;
        return Ok(st.data.entries.clone());
    }

    let raw = fs::read_to_string(&st.path).map_err(|e| e.to_string())?;
    let vault: EncryptedVault = serde_json::from_str(&raw).map_err(|e| e.to_string())?;

    let salt = B64.decode(&vault.salt).map_err(|e| e.to_string())?;
    let key = crypto::derive_key(&password, &salt).map_err(|e| e.to_string())?;

    let plaintext = crypto::decrypt(&key, &vault.data).map_err(|_| "Wrong password".to_string())?;
    let data: VaultData = serde_json::from_slice(&plaintext).map_err(|e| e.to_string())?;

    st.key = Some(key);
    st.data = data;
    Ok(st.data.entries.clone())
}

#[tauri::command]
pub fn lock_vault(state: State<'_, AppState>) -> Result<(), String> {
    let mut st = state.0.lock().map_err(|e| e.to_string())?;
    if let Some(mut key) = st.key.take() {
        key.zeroize();
    }
    st.data = VaultData::default();
    Ok(())
}

#[tauri::command]
pub fn list_entries(state: State<'_, AppState>) -> Result<Vec<PasswordEntry>, String> {
    let st = state.0.lock().map_err(|e| e.to_string())?;
    if st.key.is_none() {
        return Err("Vault is locked".into());
    }
    Ok(st.data.entries.clone())
}

#[tauri::command]
pub fn add_entry(
    title: String,
    username: String,
    password: String,
    url: Option<String>,
    notes: Option<String>,
    category: Option<String>,
    state: State<'_, AppState>,
) -> Result<PasswordEntry, String> {
    let mut st = state.0.lock().map_err(|e| e.to_string())?;
    if st.key.is_none() {
        return Err("Vault is locked".into());
    }

    let now = chrono_now();
    let entry = PasswordEntry {
        id: Uuid::new_v4().to_string(),
        title,
        username,
        password,
        url,
        notes,
        favorite: false,
        category,
        created_at: now,
        updated_at: now,
    };

    st.data.entries.push(entry.clone());
    persist(&st, st.key.as_ref().unwrap())?;
    Ok(entry)
}

#[tauri::command]
pub fn update_entry(entry: PasswordEntry, state: State<'_, AppState>) -> Result<(), String> {
    let mut st = state.0.lock().map_err(|e| e.to_string())?;
    if st.key.is_none() {
        return Err("Vault is locked".into());
    }

    if let Some(existing) = st.data.entries.iter_mut().find(|e| e.id == entry.id) {
        *existing = entry;
        existing.updated_at = chrono_now();
    } else {
        return Err("Entry not found".into());
    }
    persist(&st, st.key.as_ref().unwrap())?;
    Ok(())
}

#[tauri::command]
pub fn delete_entry(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut st = state.0.lock().map_err(|e| e.to_string())?;
    if st.key.is_none() {
        return Err("Vault is locked".into());
    }

    st.data.entries.retain(|e| e.id != id);
    persist(&st, st.key.as_ref().unwrap())?;
    Ok(())
}

/// Cryptographically secure password / passphrase generator.
///
/// - Uses `OsRng` (OS entropy: /dev/urandom, BCryptGenRandom, etc.)
/// - Random mode follows NIST SP 800-63B spirit (min 12 chars, mixed charset)
/// - Passphrase mode: Diceware-style (4–8 words from a curated list)
#[tauri::command]
pub fn generate_password(
    mode: Option<String>,
    length: Option<usize>,
    symbols: Option<bool>,
    exclude_ambiguous: Option<bool>,
) -> String {
    use rand::RngCore;
    use rand::rngs::OsRng;

    let mode = mode.unwrap_or_else(|| "password".into());
    let symbols = symbols.unwrap_or(true);
    let exclude_ambiguous = exclude_ambiguous.unwrap_or(false);

    if mode == "passphrase" {
        let word_count = length.unwrap_or(5).clamp(4, 10);
        return generate_passphrase(word_count);
    }

    // Random password mode
    let len = length.unwrap_or(20).clamp(12, 128); // NIST-aligned minimum

    let mut charset = String::from("abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ23456789");
    // Default already excludes some ambiguous (0/O, 1/l/I) for better usability.
    // If exclude_ambiguous is false we add the rest back for maximum entropy.
    if !exclude_ambiguous {
        charset.push_str("0OlI1");
    }
    if symbols {
        charset.push_str("!@#$%^&*()-_=+[]{}|;:,.<>?");
    }

    let chars: Vec<char> = charset.chars().collect();
    let mut rng = OsRng;
    let mut result = String::with_capacity(len);

    for _ in 0..len {
        let idx = (rng.next_u32() as usize) % chars.len();
        result.push(chars[idx]);
    }
    result
}

fn generate_passphrase(word_count: usize) -> String {
    use rand::RngCore;
    use rand::rngs::OsRng;

    // Compact high-quality wordlist (EFF short-list inspired, ~7776 max entropy possible).
    // Curated for memorability + uniqueness. Enough for strong 5–6 word passphrases.
    const WORDLIST: &[&str] = &[
        "able", "acid", "aged", "also", "area", "army", "away", "baby", "back", "ball",
        "band", "bank", "base", "bath", "bear", "beat", "been", "bell", "belt", "best",
        "bird", "blow", "blue", "boat", "body", "bomb", "bond", "bone", "book", "boom",
        "born", "boss", "both", "bowl", "bulk", "burn", "bush", "busy", "call", "calm",
        "came", "camp", "card", "care", "case", "cash", "cast", "cave", "cell", "chat",
        "chip", "city", "clam", "clan", "clap", "claw", "clay", "clip", "club", "clue",
        "coal", "coat", "code", "coil", "coin", "cold", "come", "cook", "cool", "cope",
        "copy", "cord", "core", "cork", "corn", "cost", "cove", "crab", "crew", "crop",
        "crow", "cube", "cult", "curb", "cure", "curl", "cute", "dale", "dame", "damp",
        "dare", "dark", "dart", "dash", "data", "date", "dawn", "dead", "deaf", "deal",
        "dean", "dear", "debt", "deck", "deep", "deer", "desk", "dial", "dice", "diet",
        "dime", "dine", "dirt", "dish", "disk", "dive", "dock", "does", "dome", "done",
        "doom", "door", "dose", "down", "doze", "drag", "draw", "drew", "drip", "drop",
        "drug", "drum", "dual", "duck", "dude", "duel", "duke", "dull", "duly", "dump",
        "dune", "dusk", "dust", "duty", "each", "earn", "ease", "east", "easy", "edge",
        "else", "even", "ever", "evil", "exam", "exit", "face", "fact", "fade", "fail",
        "fair", "fall", "fame", "fare", "farm", "fast", "fate", "fear", "feat", "feed",
        "feel", "feet", "fell", "felt", "file", "fill", "film", "find", "fine", "fire",
        "firm", "fish", "fist", "five", "flag", "flat", "fled", "flew", "flex", "flip",
        "flow", "foam", "fold", "folk", "fond", "font", "food", "fool", "foot", "ford",
        "fork", "form", "fort", "foul", "four", "free", "frog", "from", "fuel", "full",
        "fund", "fury", "fuse", "fuss", "gain", "gale", "game", "gang", "gate", "gave",
        "gaze", "gear", "gene", "gift", "girl", "give", "glad", "glow", "glue", "goal",
        "goat", "goes", "gold", "golf", "gone", "good", "gore", "gosh", "gown", "grab",
        "grad", "gram", "gray", "grew", "grid", "grim", "grin", "grip", "grit", "grow",
        "gulf", "guru", "gust", "hair", "half", "hall", "halt", "hand", "hang", "hard",
        "harm", "harp", "hate", "have", "hawk", "haze", "head", "heal", "heap", "hear",
        "heat", "heel", "held", "hell", "helm", "help", "herb", "herd", "here", "hero",
        "hers", "hide", "high", "hike", "hill", "hind", "hint", "hire", "hold", "hole",
        "holy", "home", "hope", "horn", "hose", "host", "hour", "huge", "hull", "hung",
        "hunt", "hurt", "hush", "hyde", "idea", "idle", "idol", "inch", "info", "into",
        "iron", "item", "jack", "jail", "jazz", "jean", "jeep", "jerk", "jest", "join",
        "joke", "jolt", "jump", "june", "jury", "just", "keen", "keep", "kept", "kick",
        "kill", "kind", "king", "kiss", "kite", "knee", "knew", "knit", "knob", "knot",
        "know", "lace", "lack", "lady", "laid", "lake", "lamb", "lame", "lamp", "land",
        "lane", "lark", "last", "late", "lava", "lawn", "lazy", "lead", "leaf", "leak",
        "lean", "leap", "left", "lend", "lens", "less", "liar", "lick", "life", "lift",
        "like", "limb", "lime", "line", "link", "lion", "list", "live", "load", "loaf",
        "loan", "lock", "loft", "logo", "lone", "long", "look", "loop", "lord", "lose",
        "loss", "lost", "loud", "love", "luck", "lump", "lung", "lure", "lurk", "lush",
        "made", "maid", "mail", "main", "make", "male", "mall", "malt", "many", "mare",
        "mark", "mars", "mask", "mass", "mate", "math", "maze", "mead", "meal", "mean",
        "meat", "meet", "melt", "memo", "mend", "menu", "mere", "mesh", "mess", "mice",
        "mild", "mile", "milk", "mill", "mind", "mine", "mint", "miss", "mist", "moan",
        "moat", "mock", "mode", "mold", "mole", "monk", "mood", "moon", "moor", "more",
        "moss", "most", "moth", "move", "much", "mule", "muse", "must", "mute", "myth",
        "nail", "name", "navy", "near", "neat", "neck", "need", "neon", "nest", "news",
        "next", "nice", "nine", "node", "none", "noon", "norm", "nose", "note", "noun",
        "nude", "numb", "oath", "obey", "odds", "odor", "okay", "once", "only", "onto",
        "open", "oral", "oslo", "oval", "oven", "over", "pace", "pack", "page", "paid",
        "pain", "pair", "pale", "palm", "pane", "park", "part", "pass", "past", "path",
        "pave", "pawn", "peak", "pear", "peat", "peck", "peel", "peer", "pelt", "perk",
        "pest", "pick", "pier", "pike", "pile", "pill", "pine", "pink", "pint", "pipe",
        "pity", "plan", "play", "plea", "plot", "plow", "ploy", "plug", "plum", "plus",
        "poem", "poet", "pole", "poll", "polo", "pond", "pony", "pool", "poor", "pope",
        "pore", "port", "pose", "post", "pour", "pray", "prep", "prey", "prop", "prow",
        "pull", "pulp", "pump", "pure", "push", "quit", "quiz", "race", "rack", "raft",
        "rage", "raid", "rail", "rain", "rake", "ramp", "rang", "rank", "rare", "rate",
        "rave", "read", "real", "reap", "rear", "reed", "reef", "reel", "rely", "rent",
        "rest", "rice", "rich", "ride", "ridge", "rift", "rigs", "ring", "riot", "ripe",
        "rise", "risk", "road", "roam", "roar", "robe", "rock", "rode", "role", "roll",
        "roof", "room", "root", "rope", "rose", "rosy", "rote", "rout", "rove", "rowdy",
        "royal", "ruby", "rude", "ruin", "rule", "rung", "rush", "rust", "ruth", "sack",
        "safe", "sage", "said", "sail", "sake", "sale", "salt", "same", "sand", "sane",
        "sang", "sank", "sash", "save", "says", "scan", "scar", "seal", "seam", "seat",
        "seed", "seek", "seem", "seen", "self", "sell", "send", "sent", "sept", "serf",
        "shed", "shin", "ship", "shirt", "shoe", "shop", "shot", "show", "shut", "sick",
        "side", "sift", "sign", "silk", "sill", "silo", "sing", "sink", "site", "size",
        "skin", "skip", "slab", "slam", "slap", "slat", "sled", "slew", "slid", "slim",
        "slip", "slit", "slot", "slow", "slug", "slum", "smog", "snap", "snow", "snug",
        "soak", "soap", "soar", "sock", "soda", "sofa", "soft", "soil", "sold", "sole",
        "solo", "some", "song", "soon", "soot", "sore", "sort", "soul", "soup", "sour",
        "span", "spar", "spat", "spin", "spit", "spot", "spur", "stab", "stag", "star",
        "stay", "stem", "step", "stew", "stir", "stop", "stow", "stub", "stud", "stun",
        "such", "suck", "suit", "sulk", "sump", "sung", "sunk", "sure", "surf", "swam",
        "swan", "swap", "swat", "sway", "swim", "swung", "tack", "tact", "tail", "take",
        "tale", "talk", "tall", "tame", "tank", "tape", "taps", "task", "taxi", "teal",
        "team", "tear", "tech", "teen", "tell", "tend", "tent", "term", "test", "text",
        "than", "that", "them", "then", "they", "thin", "this", "thou", "thus", "tick",
        "tide", "tidy", "tied", "tier", "tile", "till", "tilt", "time", "tiny", "tire",
        "toad", "toil", "told", "toll", "tomb", "tone", "took", "tool", "tops", "tore",
        "torn", "tort", "toss", "tour", "town", "trap", "tray", "tree", "trek", "trim",
        "trio", "trip", "trod", "trot", "true", "tube", "tuck", "tuna", "tune", "turf",
        "turn", "tusk", "twig", "twin", "type", "ugly", "undo", "unit", "unto", "upon",
        "urge", "used", "user", "vain", "vale", "vane", "vary", "vase", "vast", "veal",
        "veil", "vein", "vent", "verb", "very", "vest", "veto", "vice", "view", "vine",
        "visa", "void", "volt", "vote", "wade", "wage", "wail", "wait", "wake", "walk",
        "wall", "wand", "want", "ward", "warm", "warn", "warp", "wary", "wash", "wasp",
        "wave", "wavy", "waxy", "ways", "weak", "wean", "wear", "weed", "week", "weep",
        "weld", "well", "went", "were", "west", "what", "when", "whip", "whom", "wide",
        "wife", "wild", "will", "wilt", "wind", "wine", "wing", "wink", "wipe", "wire",
        "wise", "wish", "with", "wolf", "womb", "wood", "wool", "word", "wore", "work",
        "worm", "worn", "wove", "wrap", "wren", "writ", "yard", "yarn", "year", "yell",
        "yoga", "yolk", "your", "zany", "zeal", "zero", "zest", "zinc", "zone", "zoom",
    ];

    let mut rng = OsRng;
    let mut words = Vec::with_capacity(word_count);

    for _ in 0..word_count {
        let idx = (rng.next_u32() as usize) % WORDLIST.len();
        words.push(WORDLIST[idx]);
    }

    words.join("-")
}

fn persist(st: &VaultState, key: &MasterKey) -> Result<(), String> {
    let plaintext = serde_json::to_vec(&st.data).map_err(|e| e.to_string())?;
    let encrypted = crypto::encrypt(key, &plaintext).map_err(|e| e.to_string())?;

    let salt = if st.path.exists() {
        let raw = fs::read_to_string(&st.path).unwrap_or_default();
        if let Ok(v) = serde_json::from_str::<EncryptedVault>(&raw) {
            v.salt
        } else {
            B64.encode(crypto::generate_salt())
        }
    } else {
        B64.encode(crypto::generate_salt())
    };

    let vault = EncryptedVault {
        version: 1,
        salt,
        data: encrypted,
    };
    ensure_parent(&st.path)?;
    let json = serde_json::to_string_pretty(&vault).map_err(|e| e.to_string())?;
    fs::write(&st.path, json).map_err(|e| e.to_string())?;
    Ok(())
}

fn chrono_now() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
