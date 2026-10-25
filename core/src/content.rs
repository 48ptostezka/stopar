//! The game content format. A game is one TOML file; see `games/galaxy/game.toml`.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Identifier of an item, clue or location, e.g. `star_chart`.
pub type Id = String;

/// A language the game can be played in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    Cs,
    En,
}

impl Lang {
    /// Every supported language, in the order the language switch shows them.
    pub const ALL: [Lang; 2] = [Lang::Cs, Lang::En];

    /// BCP 47 code, as used in `<html lang>`.
    pub fn code(self) -> &'static str {
        match self {
            Lang::Cs => "cs",
            Lang::En => "en",
        }
    }

    /// Reads a browser language such as `cs-CZ` or `en-GB`.
    pub fn from_code(code: &str) -> Option<Lang> {
        let code = code.to_ascii_lowercase();
        Lang::ALL.into_iter().find(|l| code == l.code() || code.starts_with(&format!("{}-", l.code())))
    }

    /// The language's name in that language, for the language switch.
    pub fn native_name(self) -> &'static str {
        match self {
            Lang::Cs => "Čeština",
            Lang::En => "English",
        }
    }
}

/// A piece of text in every supported language.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Text {
    pub cs: String,
    pub en: String,
}

impl Text {
    pub fn get(&self, lang: Lang) -> &str {
        match lang {
            Lang::Cs => &self.cs,
            Lang::En => &self.en,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (Lang, &str)> {
        Lang::ALL.into_iter().map(move |l| (l, self.get(l)))
    }
}

/// A whole game: its settings and everything a player can find.
///
/// Tables keep the order they have in the file, so the inventory lists
/// items in the order the author wrote them.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Game {
    pub game: GameInfo,
    #[serde(default)]
    pub items: IndexMap<Id, Item>,
    #[serde(default)]
    pub clues: IndexMap<Id, Clue>,
    #[serde(default)]
    pub locations: IndexMap<Id, Location>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameInfo {
    /// Short machine name. Part of every QR code, so don't change it once codes are printed.
    pub id: Id,
    /// Folder name under `app/skins/`.
    pub skin: String,
    pub title: Text,
    /// Used when the phone's language is neither Czech nor English.
    pub default_language: Lang,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Item {
    pub name: Text,
    pub description: Text,
    #[serde(default)]
    pub icon: ItemIcon,
}

/// An item's picture: a built-in icon (`icon = "key"`), or an SVG file from
/// the game's `icons/` folder (`icon = "wheel.svg"`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ItemIcon {
    Builtin(Icon),
    File(String),
}

impl Default for ItemIcon {
    fn default() -> ItemIcon {
        ItemIcon::Builtin(Icon::Generic)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Clue {
    pub title: Text,
    pub text: Text,
}

/// A place with a QR code. Its story is a small graph of scenes: a first
/// visit starts at `start` (or the first scene), and the player moves on with
/// Continue, a choice or a password.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Location {
    /// The unique part of this location's QR code, e.g. `7Q4M-2KD9`.
    pub code: String,
    pub name: Text,
    /// The scene a first visit opens at. Defaults to the first scene in the file.
    #[serde(default)]
    pub start: Option<Id>,
    /// The screens of this location's story, in file order.
    pub scenes: IndexMap<Id, Scene>,
}

impl Location {
    pub fn start_scene(&self) -> Option<&Id> {
        self.start.as_ref().or_else(|| self.scenes.keys().next())
    }
}

/// One screen of a location's story.
///
/// It shows an optional picture, then `text`, then every paragraph whose
/// condition holds. It ends in at most one way: `next` (a Continue key),
/// `choices`, or `password`. A scene with none of them ends the story here.
/// With `requires`, that way on is offered only once the condition holds.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scene {
    /// A file in `games/<game>/images/` (png, svg, jpg or webp).
    #[serde(default)]
    pub image: Option<String>,
    /// Describes the picture for players who can't see it.
    #[serde(default)]
    pub alt: Option<Text>,
    #[serde(default)]
    pub text: Option<Text>,
    #[serde(default)]
    pub paragraphs: Vec<Paragraph>,
    /// Given the first time the player reaches this scene.
    #[serde(default)]
    pub give: Rewards,
    /// Flags set the first time the player reaches this scene.
    #[serde(default)]
    pub set: Vec<String>,
    #[serde(default)]
    pub next: Option<Id>,
    #[serde(default)]
    pub choices: Vec<Choice>,
    /// `one` (default): the first choice made is final. `any`: every choice stays open.
    #[serde(default)]
    pub pick: Pick,
    #[serde(default)]
    pub password: Option<Password>,
    /// The way on (Continue, choices or password) is offered only while this
    /// holds. Until then the player sees the scene and `waiting`, and Done.
    #[serde(default)]
    pub requires: Condition,
    /// Shown while `requires` doesn't hold, e.g. "Come back with a power cell".
    #[serde(default)]
    pub waiting: Option<Text>,
    /// Silences the build warning about a final decision whose options need
    /// different things, when losing an option is intended.
    #[serde(default)]
    pub allow_lockout: bool,
}

impl Scene {
    /// True if the scene leads somewhere; false for a scene that ends the story.
    pub fn has_exit(&self) -> bool {
        self.next.is_some() || !self.choices.is_empty() || self.password.is_some()
    }
}

impl Condition {
    pub fn is_empty(&self) -> bool {
        self == &Condition::default()
    }
}

/// Text shown only while its condition holds.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Paragraph {
    #[serde(default)]
    pub when: Condition,
    pub text: Text,
}

/// Everything listed must hold. An empty condition always holds.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Condition {
    /// Item id to the least quantity held.
    pub items: IndexMap<Id, u32>,
    pub clues: Vec<Id>,
    pub flags: Vec<String>,
    /// Locations already scanned.
    pub visited: Vec<Id>,
    pub without_items: Vec<Id>,
    pub without_clues: Vec<Id>,
    pub without_flags: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Rewards {
    pub items: IndexMap<Id, u32>,
    pub clues: Vec<Id>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Pick {
    #[default]
    One,
    Any,
}

/// One option of a decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    /// Stable name, saved in progress. Unique within the scene.
    pub id: Id,
    pub label: Text,
    /// When this doesn't hold, the option is hidden, or shown disabled with `locked`.
    #[serde(default)]
    pub when: Condition,
    /// Shown on a disabled option while it isn't available, e.g. "You need a key".
    #[serde(default)]
    pub locked: Option<Text>,
    /// Items used up by this option. The option is available only while they are held.
    #[serde(default)]
    pub take: IndexMap<Id, u32>,
    #[serde(default)]
    pub give: Rewards,
    #[serde(default)]
    pub set: Vec<String>,
    /// The scene this option leads to. Without it, the story ends here.
    #[serde(default)]
    pub to: Option<Id>,
}

/// A text field the player must fill in correctly to go on.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Password {
    /// Accepted answers. Letter case, accents, spaces and punctuation are ignored.
    pub answers: Vec<String>,
    /// Label of the field; defaults to "Password".
    #[serde(default)]
    pub prompt: Option<Text>,
    #[serde(default)]
    pub hint: Option<Text>,
    /// Shown after a wrong answer; defaults to "That's not it".
    #[serde(default)]
    pub wrong: Option<Text>,
    #[serde(default)]
    pub give: Rewards,
    #[serde(default)]
    pub set: Vec<String>,
    /// The scene a correct answer leads to. Without it, the story ends here.
    #[serde(default)]
    pub to: Option<Id>,
}

impl Password {
    pub fn accepts(&self, input: &str) -> bool {
        let input = normalize_answer(input);
        !input.is_empty() && self.answers.iter().any(|a| normalize_answer(a) == input)
    }
}

/// Lowercase, without accents, keeping only letters and digits:
/// "Vega-3", "vega 3" and "VEGA3" all become "vega3"; "Říp" becomes "rip".
pub fn normalize_answer(text: &str) -> String {
    text.chars()
        .flat_map(char::to_lowercase)
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ä' | 'ã' | 'å' | 'ą' => 'a',
            'č' | 'ç' | 'ć' => 'c',
            'ď' => 'd',
            'é' | 'ě' | 'è' | 'ê' | 'ë' | 'ę' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ľ' | 'ĺ' | 'ł' => 'l',
            'ň' | 'ñ' | 'ń' => 'n',
            'ó' | 'ò' | 'ô' | 'ö' | 'õ' | 'ő' => 'o',
            'ř' | 'ŕ' => 'r',
            'š' | 'ś' => 's',
            'ť' => 't',
            'ú' | 'ů' | 'ù' | 'û' | 'ü' | 'ű' => 'u',
            'ý' | 'ÿ' => 'y',
            'ž' | 'ź' | 'ż' => 'z',
            other => other,
        })
        .filter(|c| c.is_alphanumeric())
        .collect()
}

/// Built-in item icons. Add a variant here and draw it in `app/src/ui/icons.rs`,
/// or give a game its own icons as SVG files (see [`ItemIcon`]).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Icon {
    #[default]
    Generic,
    Map,
    Key,
    Coin,
    Lantern,
    Feather,
    Cell,
    Crystal,
    Decoder,
    Tank,
    Letter,
    Flask,
}

impl Game {
    /// Parses a game file. Call [`Game::check`] afterwards.
    #[cfg(feature = "toml")]
    pub fn from_toml(text: &str) -> Result<Game, String> {
        toml::from_str(text).map_err(|e| e.to_string())
    }

    pub fn scene(&self, location: &str, scene: &str) -> Option<&Scene> {
        self.locations.get(location)?.scenes.get(scene)
    }

    /// Every picture the game uses, as file names in `games/<game>/images/`.
    pub fn images(&self) -> Vec<&str> {
        let mut files: Vec<&str> = self
            .locations
            .values()
            .flat_map(|l| l.scenes.values())
            .filter_map(|s| s.image.as_deref())
            .collect();
        files.sort_unstable();
        files.dedup();
        files
    }

    /// Every icon file the game's items use, as file names in `games/<game>/icons/`.
    pub fn icon_files(&self) -> Vec<&str> {
        let mut files: Vec<&str> = self
            .items
            .values()
            .filter_map(|i| match &i.icon {
                ItemIcon::File(f) => Some(f.as_str()),
                ItemIcon::Builtin(_) => None,
            })
            .collect();
        files.sort_unstable();
        files.dedup();
        files
    }

    /// Finds the location whose QR code carries `code`.
    pub fn location_by_code(&self, code: &str) -> Option<(&Id, &Location)> {
        self.locations.iter().find(|(_, l)| l.code == code)
    }
}
