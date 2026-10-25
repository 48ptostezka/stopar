//! Every interface string, in every language.
//!
//! Game text (names, stories, clues) lives in the game file instead.
//! To add a language: add it to `stopa_core::Lang`, then add a `Ui` value here.
//! The compiler lists everything that still needs a translation.

use stopa_core::Lang;

pub struct Ui {
    // Menu and page titles. Skins decide the letter case.
    pub scan: &'static str,
    pub inventory: &'static str,
    pub logbook: &'static str,
    pub settings: &'static str,
    pub menu: &'static str,
    pub back: &'static str,

    // Scan
    pub camera_starting: &'static str,
    pub looking: &'static str,
    pub scan_hint: &'static str,
    pub code_unknown: &'static str,
    pub code_other_game: &'static str,
    pub camera_denied: (&'static str, &'static str),
    pub camera_missing: (&'static str, &'static str),
    pub camera_insecure: (&'static str, &'static str),
    pub camera_failed: (&'static str, &'static str),
    pub try_again: &'static str,

    // Location
    pub you_found: &'static str,
    pub you_used: &'static str,
    pub next: &'static str,
    pub done: &'static str,
    pub password: &'static str,
    pub check: &'static str,
    pub wrong_answer: &'static str,
    pub correct: &'static str,
    pub waiting: &'static str,

    // Inventory
    pub column_item: &'static str,
    pub column_quantity: &'static str,
    pub inventory_empty: &'static str,

    // Logbook
    pub logbook_empty: &'static str,
    pub clue_new: &'static str,

    // Settings
    pub language: &'static str,
    pub offline: &'static str,
    pub offline_ready: &'static str,
    pub offline_preparing: &'static str,
    pub offline_insecure: &'static str,
    pub offline_unsupported: &'static str,
    pub offline_dev: &'static str,
    pub developer: &'static str,
    pub reset_progress: &'static str,
    pub reset_confirm: &'static str,
    pub reset_note: &'static str,
    pub delete_offline: &'static str,
    pub delete_offline_confirm: &'static str,
    pub delete_offline_note: &'static str,
    pub delete_offline_working: &'static str,
    pub needs_network: &'static str,
}

pub const CS: Ui = Ui {
    scan: "Skenovat",
    inventory: "Inventář",
    logbook: "Deník",
    settings: "Nastavení",
    menu: "Hlavní nabídka",
    back: "Zpět",

    camera_starting: "Zapínám fotoaparát…",
    looking: "Hledám kód…",
    scan_hint: "Namiř fotoaparát na QR kód na tomto místě.",
    code_unknown: "Tento kód do hry nepatří.",
    code_other_game: "Tento kód patří k jiné hře.",
    camera_denied: (
        "Přístup k fotoaparátu je zablokovaný",
        "Povol této stránce fotoaparát v nastavení prohlížeče a pak klepni na Zkusit znovu.",
    ),
    camera_missing: (
        "Fotoaparát nebyl nalezen",
        "Ke skenování je potřeba telefon nebo tablet s fotoaparátem.",
    ),
    camera_insecure: (
        "Skenování potřebuje zabezpečené připojení",
        "Otevři hru přes adresu začínající https://.",
    ),
    camera_failed: (
        "Fotoaparát se nespustil",
        "Zavři ostatní aplikace, které fotoaparát používají, a zkus to znovu.",
    ),
    try_again: "Zkusit znovu",

    you_found: "Získáno",
    you_used: "Použito",
    next: "Pokračovat",
    done: "Hotovo",
    password: "Heslo",
    check: "Ověřit",
    wrong_answer: "To není ono. Zkus to znovu.",
    correct: "Správně!",
    waiting: "Tady zatím dál nemůžeš. Vrať se později.",

    column_item: "Předmět",
    column_quantity: "Ks",
    inventory_empty: "Inventář je prázdný. Předměty získáš naskenováním kódů na místech hry.",

    logbook_empty: "Zatím žádné stopy. Každé naskenované místo sem může nějakou přidat.",
    clue_new: "Nová",

    language: "Jazyk",
    offline: "Hra bez internetu",
    offline_ready: "Připraveno. Hra funguje i bez připojení.",
    offline_preparing: "Připravuje se. Nech hru chvíli otevřenou s připojením a pak ji otevři znovu.",
    offline_insecure: "Nedostupné: hra je otevřená přes http. Bez internetu funguje jen přes https.",
    offline_unsupported: "Tento prohlížeč hru bez internetu nepodporuje.",
    offline_dev: "Vypnuto ve vývojovém sestavení (trunk serve).",
    developer: "Pro vývojáře",
    reset_progress: "Smazat postup",
    reset_confirm: "Klepni znovu a postup se smaže",
    reset_note: "Smaže předměty, stopy, rozhodnutí a místa v příbězích. Jazyk zůstane.",
    delete_offline: "Smazat offline soubory",
    delete_offline_confirm: "Klepni znovu a soubory se smažou",
    delete_offline_note:
        "Smaže kopii hry uloženou pro hraní bez internetu a načte ji znovu ze sítě. Postup zůstane.",
    delete_offline_working: "Mažu a načítám znovu…",
    needs_network: "Teď nejsi připojený. Bez internetu by se hra znovu nenačetla.",
};

pub const EN: Ui = Ui {
    scan: "Scan",
    inventory: "Inventory",
    logbook: "Logbook",
    settings: "Settings",
    menu: "Main menu",
    back: "Back",

    camera_starting: "Starting the camera…",
    looking: "Looking for a code…",
    scan_hint: "Point the camera at the QR code at this place.",
    code_unknown: "This code isn't part of the game.",
    code_other_game: "This code belongs to a different game.",
    camera_denied: (
        "Camera access is blocked",
        "Allow this page to use the camera in your browser's settings, then tap Try again.",
    ),
    camera_missing: ("No camera found", "Scanning needs a phone or tablet with a camera."),
    camera_insecure: (
        "Scanning needs a secure connection",
        "Open the game at an address starting with https://.",
    ),
    camera_failed: ("The camera didn't start", "Close other apps that are using the camera, then try again."),
    try_again: "Try again",

    you_found: "You found",
    you_used: "Used",
    next: "Continue",
    done: "Done",
    password: "Password",
    check: "Check",
    wrong_answer: "That's not it. Try again.",
    correct: "Correct!",
    waiting: "You can't go on from here yet. Come back later.",

    column_item: "Item",
    column_quantity: "Qty",
    inventory_empty: "Your inventory is empty. Scan the codes at places in the game to collect items.",

    logbook_empty: "No clues yet. Each place you scan can add some here.",
    clue_new: "New",

    language: "Language",
    offline: "Offline play",
    offline_ready: "Ready. The game works without a connection.",
    offline_preparing: "Getting ready. Keep the game open online for a moment, then open it again.",
    offline_insecure: "Not available: the game was opened over http. Offline play needs https.",
    offline_unsupported: "This browser doesn't support offline play.",
    offline_dev: "Off in development builds (trunk serve).",
    developer: "For developers",
    reset_progress: "Reset progress",
    reset_confirm: "Tap again to delete all progress",
    reset_note: "Deletes items, clues, decisions and places in stories. The language stays.",
    delete_offline: "Delete offline files",
    delete_offline_confirm: "Tap again to delete the files",
    delete_offline_note: "Deletes the copy of the game kept for offline play and loads it again from the network. Progress stays.",
    delete_offline_working: "Deleting and reloading…",
    needs_network: "You're offline right now. Without a connection the game couldn't load again.",
};

pub fn ui(lang: Lang) -> &'static Ui {
    match lang {
        Lang::Cs => &CS,
        Lang::En => &EN,
    }
}

/// "Found at Relay station K-7" / "Nalezeno: Reléová stanice K-7"
pub fn found_at(lang: Lang, places: &str) -> String {
    match lang {
        Lang::Cs => format!("Nalezeno: {places}"),
        Lang::En => format!("Found at {places}"),
    }
}
