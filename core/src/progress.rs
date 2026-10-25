//! What a player has done: places visited, items held, clues found, decisions
//! made, passwords solved, and where they are in each location's story.

use std::collections::{BTreeMap, BTreeSet};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::content::{Choice, Condition, Game, Id, Location, Pick, Rewards, Scene};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Progress {
    /// Location ids, in the order they were first scanned.
    pub visited: Vec<Id>,
    /// Item id to quantity held. Items used up to zero are removed.
    pub items: IndexMap<Id, u32>,
    /// Clue ids, in the order they were found.
    pub clues: Vec<Id>,
    /// Clues found but not opened yet in the logbook.
    pub unread: BTreeSet<Id>,
    /// Story flags set by scenes, choices and passwords. Never shown to players.
    pub flags: BTreeSet<String>,
    /// Story state per location.
    pub places: IndexMap<Id, Place>,
    /// Where each item and clue was obtained (location ids), for "Found at".
    pub item_sources: IndexMap<Id, Vec<Id>>,
    pub clue_sources: IndexMap<Id, Vec<Id>>,
}

/// A player's state in one location's story.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Place {
    /// The scene to open on the next scan.
    pub scene: Id,
    /// Scenes reached at least once. Only these can be opened again.
    pub seen: BTreeSet<Id>,
    /// Scene id to the choice ids taken there.
    pub chosen: BTreeMap<Id, BTreeSet<Id>>,
    /// Scenes whose password was answered.
    pub solved: BTreeSet<Id>,
}

/// What changed in the player's hands because of one step in a story.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Found {
    pub items: Vec<(Id, u32)>,
    pub clues: Vec<Id>,
    pub used: Vec<(Id, u32)>,
}

impl Found {
    pub fn is_empty(&self) -> bool {
        self.items.is_empty() && self.clues.is_empty() && self.used.is_empty()
    }

    pub fn merge(&mut self, other: Found) {
        self.items.extend(other.items);
        self.clues.extend(other.clues);
        self.used.extend(other.used);
    }
}

/// The result of one step in a story: a scan, Continue, a choice or a password.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// The scene now open (already entered), or `None` if the story ends on
    /// the scene where the step was taken.
    pub next: Option<Id>,
    /// What the step and the scene it led to gave and used up. Empty when
    /// both had already been done before.
    pub found: Found,
}

/// How a choice should be shown right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChoiceStatus {
    /// Can be picked.
    Open,
    /// Shown disabled with its `locked` text: its condition doesn't hold.
    Locked,
    /// Not shown: its condition doesn't hold and it has no `locked` text.
    Hidden,
    /// Already picked. Picking it again follows it without new effects.
    Chosen,
    /// Another option of this final decision was picked.
    Closed,
}

impl Progress {
    pub fn has_visited(&self, location: &str) -> bool {
        self.visited.iter().any(|v| v == location)
    }

    pub fn count(&self, item: &str) -> u32 {
        self.items.get(item).copied().unwrap_or(0)
    }

    pub fn has_clue(&self, clue: &str) -> bool {
        self.clues.iter().any(|c| c == clue)
    }

    pub fn has_seen(&self, location: &str, scene: &str) -> bool {
        self.places.get(location).is_some_and(|p| p.seen.contains(scene))
    }

    pub fn is_solved(&self, location: &str, scene: &str) -> bool {
        self.places.get(location).is_some_and(|p| p.solved.contains(scene))
    }

    pub fn meets(&self, c: &Condition) -> bool {
        c.items.iter().all(|(id, n)| self.count(id) >= *n)
            && c.clues.iter().all(|id| self.has_clue(id))
            && c.flags.iter().all(|f| self.flags.contains(f))
            && c.visited.iter().all(|l| self.has_visited(l))
            && c.without_items.iter().all(|id| self.count(id) == 0)
            && c.without_clues.iter().all(|id| !self.has_clue(id))
            && c.without_flags.iter().all(|f| !self.flags.contains(f))
    }

    // Story steps.
    //
    // A player moves through a story only by these four steps, and each one
    // checks that it is allowed from where the player is: the scene must have
    // been reached by playing, its `requires` must hold, and the step must be
    // one the scene offers. So rewards can't be collected by opening a scene
    // directly (a typed address, a stale page in another tab), and a final
    // decision pays out once, for one option.

    /// Records a scan and opens the location's story where the player left
    /// off, or at the start if they finished it there (or never began it).
    /// Returns `None` if the game has no such location.
    pub fn scan(&mut self, game: &Game, location: &str) -> Option<Outcome> {
        let loc = game.locations.get(location)?;
        let start = loc.start_scene()?.clone();
        if !self.has_visited(location) {
            self.visited.push(location.to_owned());
        }
        let place = self.places.entry(location.to_owned()).or_default();
        let resume = match loc.scenes.get(&place.scene) {
            Some(scene) if scene.has_exit() && place.seen.contains(&place.scene) => place.scene.clone(),
            _ => start,
        };
        let found = self.enter(game, location, &resume);
        Some(Outcome { next: Some(resume), found })
    }

    /// The Continue key: follows the scene's `next`, or the way on from a
    /// password the player already solved. Returns `None` if the scene offers
    /// neither right now.
    pub fn continue_on(&mut self, game: &Game, location: &str, scene_id: &str) -> Option<Outcome> {
        let scene = self.playable(game, location, scene_id)?;
        let to = match (&scene.next, &scene.password) {
            (Some(next), _) => next,
            (None, Some(pw)) if self.is_solved(location, scene_id) => pw.to.as_ref()?,
            _ => return None,
        };
        let found = self.enter(game, location, to);
        Some(Outcome { next: Some(to.clone()), found })
    }

    /// Picks an option. An option already picked can be followed again, with
    /// no new rewards. Returns `None` if the option isn't available.
    pub fn choose(
        &mut self,
        game: &Game,
        location: &str,
        scene_id: &str,
        choice_id: &str,
    ) -> Option<Outcome> {
        let scene = self.playable(game, location, scene_id)?;
        let choice = scene.choices.iter().find(|c| c.id == choice_id)?;
        let mut found = match self.choice_status(location, scene_id, scene, choice) {
            ChoiceStatus::Chosen => Found::default(),
            ChoiceStatus::Open => {
                self.places
                    .entry(location.to_owned())
                    .or_default()
                    .chosen
                    .entry(scene_id.to_owned())
                    .or_default()
                    .insert(choice.id.clone());
                self.apply(location, &choice.give, &choice.take, &choice.set)
            }
            ChoiceStatus::Locked | ChoiceStatus::Hidden | ChoiceStatus::Closed => return None,
        };
        Some(self.follow(game, location, choice.to.as_ref(), &mut found))
    }

    /// Checks a password. Returns `None` for a wrong answer, or if the scene
    /// has no password the player can answer right now.
    pub fn answer(&mut self, game: &Game, location: &str, scene_id: &str, input: &str) -> Option<Outcome> {
        let password = self.playable(game, location, scene_id)?.password.as_ref()?;
        let mut found = if self.is_solved(location, scene_id) {
            Found::default()
        } else if password.accepts(input) {
            self.places.entry(location.to_owned()).or_default().solved.insert(scene_id.to_owned());
            self.apply(location, &password.give, &IndexMap::new(), &password.set)
        } else {
            return None;
        };
        Some(self.follow(game, location, password.to.as_ref(), &mut found))
    }

    /// True while the scene's `requires` holds, so its way on is offered.
    pub fn is_ready(&self, scene: &Scene) -> bool {
        self.meets(&scene.requires)
    }

    /// The scene, if the player may take a step from it right now: reached by
    /// playing, and its `requires` holds.
    fn playable<'g>(&self, game: &'g Game, location: &str, scene_id: &str) -> Option<&'g Scene> {
        let scene = game.scene(location, scene_id)?;
        (self.has_seen(location, scene_id) && self.is_ready(scene)).then_some(scene)
    }

    /// Opens `to` after a choice or password, adding its rewards to `found`.
    fn follow(&mut self, game: &Game, location: &str, to: Option<&Id>, found: &mut Found) -> Outcome {
        if let Some(to) = to {
            found.merge(self.enter(game, location, to));
        }
        Outcome { next: to.cloned(), found: std::mem::take(found) }
    }

    /// Opens a scene. Its rewards and flags apply the first time only.
    /// Private: callers reach scenes only through the steps above.
    fn enter(&mut self, game: &Game, location: &str, scene: &str) -> Found {
        let Some(s) = game.scene(location, scene) else { return Found::default() };
        let first_time = {
            let place = self.places.entry(location.to_owned()).or_default();
            place.scene = scene.to_owned();
            place.seen.insert(scene.to_owned())
        };
        if first_time {
            self.apply(location, &s.give, &IndexMap::new(), &s.set)
        } else {
            Found::default()
        }
    }

    pub fn choice_status(
        &self,
        location: &str,
        scene_id: &str,
        scene: &Scene,
        choice: &Choice,
    ) -> ChoiceStatus {
        let chosen = self.places.get(location).and_then(|p| p.chosen.get(scene_id));
        let available = self.meets(&choice.when) && choice.take.iter().all(|(id, n)| self.count(id) >= *n);
        if chosen.is_some_and(|c| c.contains(&choice.id)) {
            ChoiceStatus::Chosen
        } else if scene.pick == Pick::One && chosen.is_some_and(|c| !c.is_empty()) {
            // Decided. A secret option the player never had stays secret.
            if available || choice.locked.is_some() {
                ChoiceStatus::Closed
            } else {
                ChoiceStatus::Hidden
            }
        } else if available {
            ChoiceStatus::Open
        } else if choice.locked.is_some() {
            ChoiceStatus::Locked
        } else {
            ChoiceStatus::Hidden
        }
    }

    fn apply(&mut self, location: &str, give: &Rewards, take: &IndexMap<Id, u32>, set: &[String]) -> Found {
        let mut found = Found::default();
        for (id, n) in take {
            let have = self.count(id);
            let used = (*n).min(have);
            if used == 0 {
                continue;
            }
            if have == used {
                self.items.shift_remove(id);
            } else {
                self.items.insert(id.clone(), have - used);
            }
            found.used.push((id.clone(), used));
        }
        for (id, n) in &give.items {
            *self.items.entry(id.clone()).or_default() += n;
            let sources = self.item_sources.entry(id.clone()).or_default();
            if !sources.iter().any(|s| s == location) {
                sources.push(location.to_owned());
            }
            found.items.push((id.clone(), *n));
        }
        for clue in &give.clues {
            if !self.has_clue(clue) {
                self.clues.push(clue.clone());
                self.unread.insert(clue.clone());
                self.clue_sources.entry(clue.clone()).or_default().push(location.to_owned());
                found.clues.push(clue.clone());
            }
        }
        self.flags.extend(set.iter().cloned());
        found
    }

    pub fn is_unread(&self, clue: &str) -> bool {
        self.unread.contains(clue)
    }

    pub fn mark_read(&mut self, clue: &str) {
        self.unread.remove(clue);
    }

    /// Locations where this item was obtained.
    pub fn where_item_found<'g>(&self, game: &'g Game, item: &str) -> Vec<&'g Location> {
        Self::sources(game, self.item_sources.get(item))
    }

    /// Locations where this clue was found.
    pub fn where_clue_found<'g>(&self, game: &'g Game, clue: &str) -> Vec<&'g Location> {
        Self::sources(game, self.clue_sources.get(clue))
    }

    fn sources<'g>(game: &'g Game, ids: Option<&Vec<Id>>) -> Vec<&'g Location> {
        ids.into_iter().flatten().filter_map(|id| game.locations.get(id)).collect()
    }

    /// Drops anything the game no longer contains, e.g. after the author renamed an item.
    pub fn retain_known(&mut self, game: &Game) {
        self.visited.retain(|id| game.locations.contains_key(id));
        self.items.retain(|id, qty| game.items.contains_key(id) && *qty > 0);
        self.clues.retain(|id| game.clues.contains_key(id));
        self.unread.retain(|id| game.clues.contains_key(id));
        self.item_sources.retain(|id, _| game.items.contains_key(id));
        self.clue_sources.retain(|id, _| game.clues.contains_key(id));
        self.places.retain(|id, _| game.locations.contains_key(id));
        for (id, place) in self.places.iter_mut() {
            let scenes = &game.locations[id].scenes;
            if !scenes.contains_key(&place.scene) {
                place.scene.clear();
            }
            place.seen.retain(|s| scenes.contains_key(s));
            place.chosen.retain(|s, _| scenes.contains_key(s));
            place.solved.retain(|s| scenes.contains_key(s));
        }
    }
}

#[cfg(all(test, feature = "toml"))]
mod tests {
    use super::*;

    fn game() -> Game {
        Game::from_toml(include_str!("fixture.toml")).unwrap()
    }

    fn status(p: &Progress, g: &Game, loc: &str, scene: &str, choice: &str) -> ChoiceStatus {
        let s = g.scene(loc, scene).unwrap();
        let c = s.choices.iter().find(|c| c.id == choice).unwrap();
        p.choice_status(loc, scene, s, c)
    }

    /// Solves the gate's password, which gives the key.
    fn get_key(p: &mut Progress, g: &Game) {
        p.scan(g, "gate");
        p.answer(g, "gate", "lock", "vega3").unwrap();
    }

    /// Plays from the door into the guard's room, using the key.
    fn reach_room(p: &mut Progress, g: &Game) {
        p.scan(g, "door");
        get_key(p, g);
        assert_eq!(p.choose(g, "door", "hall", "open").unwrap().next.as_deref(), Some("room"));
    }

    /// Everything a player holds, for comparing before and after.
    fn holdings(p: &Progress) -> (IndexMap<Id, u32>, Vec<Id>, BTreeSet<String>) {
        (p.items.clone(), p.clues.clone(), p.flags.clone())
    }

    #[test]
    fn scene_rewards_are_given_once() {
        let g = game();
        let mut p = Progress::default();
        let first = p.scan(&g, "door").unwrap();
        assert_eq!(first.next.as_deref(), Some("hall"));
        assert_eq!(first.found.items, vec![("coin".into(), 2)]);
        assert!(p.scan(&g, "door").unwrap().found.is_empty());
        assert_eq!(p.count("coin"), 2);
    }

    #[test]
    fn conditions_decide_what_is_shown() {
        let g = game();
        let mut p = Progress::default();
        p.scan(&g, "door");
        let locked_para = &g.scene("door", "hall").unwrap().paragraphs[0].when;
        assert!(p.meets(locked_para));
        assert_eq!(status(&p, &g, "door", "hall", "open"), ChoiceStatus::Locked);
        assert_eq!(status(&p, &g, "door", "hall", "secret"), ChoiceStatus::Hidden);
        assert_eq!(status(&p, &g, "door", "hall", "knock"), ChoiceStatus::Open);
        assert!(p.choose(&g, "door", "hall", "open").is_none());
    }

    #[test]
    fn passwords_ignore_case_accents_and_punctuation() {
        let g = game();
        let mut p = Progress::default();
        p.scan(&g, "gate");
        assert!(p.answer(&g, "gate", "lock", "orion").is_none());
        assert!(p.continue_on(&g, "gate", "lock").is_none(), "no way on before solving");
        let first = p.answer(&g, "gate", "lock", "  vega 3 ").unwrap();
        assert_eq!(first.next.as_deref(), Some("open"));
        assert_eq!(first.found.items, vec![("key".into(), 1)]);
        let again = p.answer(&g, "gate", "lock", "anything").unwrap();
        assert!(again.found.is_empty());
        let onward = p.continue_on(&g, "gate", "lock").unwrap();
        assert_eq!(onward.next.as_deref(), Some("open"));
        assert!(onward.found.is_empty());
        assert_eq!(p.count("key"), 1);
        assert!(crate::content::normalize_answer("Říp") == "rip");
    }

    #[test]
    fn pick_any_keeps_options_open_and_each_rewards_once() {
        let g = game();
        let mut p = Progress::default();
        p.scan(&g, "door");
        assert_eq!(p.choose(&g, "door", "hall", "knock").unwrap().found.clues, vec!["knock".to_string()]);
        assert!(p.choose(&g, "door", "hall", "knock").unwrap().found.is_empty());
        get_key(&mut p, &g);
        assert_eq!(status(&p, &g, "door", "hall", "open"), ChoiceStatus::Open);
        let open = p.choose(&g, "door", "hall", "open").unwrap();
        assert_eq!(open.found.used, vec![("key".into(), 1)]);
        assert_eq!(p.count("key"), 0);
        assert!(!p.items.contains_key("key"));
        // Having used the key, the option still counts as taken.
        assert_eq!(status(&p, &g, "door", "hall", "open"), ChoiceStatus::Chosen);
    }

    #[test]
    fn pick_one_is_final_and_sets_flags() {
        let g = game();
        let mut p = Progress::default();
        reach_room(&mut p, &g);
        let paid = p.choose(&g, "door", "room", "pay").unwrap();
        assert_eq!(paid.found.used, vec![("coin".into(), 1)]);
        assert_eq!(status(&p, &g, "door", "room", "leave"), ChoiceStatus::Closed);
        assert!(p.choose(&g, "door", "room", "leave").is_none());
        assert!(p.flags.contains("friend"));
        assert_eq!(status(&p, &g, "door", "hall", "secret"), ChoiceStatus::Open);
    }

    /// The rule the whole story model rests on: a final decision pays out
    /// once, for one option, whatever route the player takes afterwards.
    #[test]
    fn a_final_decision_pays_out_once_by_any_route() {
        let g = game();
        let mut p = Progress::default();
        reach_room(&mut p, &g);
        p.choose(&g, "door", "hall", "knock").unwrap(); // the menu's own reward, before deciding
        p.choose(&g, "door", "room", "pay").unwrap();
        let after_decision = holdings(&p);

        // The other option, directly.
        assert!(p.choose(&g, "door", "room", "leave").is_none());
        // Back to the decision by another way (the secret entrance the
        // decision itself opened), then both options again.
        assert_eq!(p.choose(&g, "door", "hall", "secret").unwrap().next.as_deref(), Some("room"));
        assert!(p.choose(&g, "door", "room", "leave").is_none());
        let again = p.choose(&g, "door", "room", "pay").unwrap();
        assert!(again.found.is_empty(), "following the chosen option again gives nothing");
        // A new scan of the place, then both options again.
        p.scan(&g, "door");
        assert!(p.choose(&g, "door", "room", "leave").is_none());
        assert!(p.choose(&g, "door", "room", "pay").unwrap().found.is_empty());
        // Every other step the place offers, once more.
        p.choose(&g, "door", "hall", "knock");
        p.continue_on(&g, "door", "end");
        assert_eq!(holdings(&p), after_decision, "nothing new after the decision");
    }

    #[test]
    fn steps_only_work_from_scenes_reached_by_playing() {
        let g = game();
        let mut p = Progress::default();
        // Nothing scanned yet: no step anywhere does anything.
        assert!(p.choose(&g, "door", "room", "leave").is_none());
        assert!(p.choose(&g, "door", "hall", "knock").is_none());
        assert!(p.answer(&g, "gate", "lock", "vega3").is_none());
        assert!(p.continue_on(&g, "gate", "lock").is_none());
        assert_eq!(p, Progress::default());
        // At the door, the guard's room still can't be used before reaching it.
        p.scan(&g, "door");
        assert!(p.choose(&g, "door", "room", "leave").is_none());
        assert!(!p.has_seen("door", "room"));
        // Unknown names do nothing.
        assert!(p.choose(&g, "door", "hall", "fly").is_none());
        assert!(p.choose(&g, "nowhere", "hall", "knock").is_none());
        assert!(p.scan(&g, "nowhere").is_none());
    }

    #[test]
    fn a_waiting_scene_offers_no_way_on_until_its_requirement_holds() {
        let g = game();
        let mut p = Progress::default();
        let arrival = p.scan(&g, "vault").unwrap();
        assert_eq!(arrival.found.clues, vec!["safe".to_string()], "the scene itself still gives");
        let safe = g.scene("vault", "safe").unwrap();
        assert!(!p.is_ready(safe));
        assert!(p.choose(&g, "vault", "safe", "gold").is_none());
        assert!(p.choose(&g, "vault", "safe", "map").is_none());
        // Scanning again returns to the waiting scene.
        assert_eq!(p.scan(&g, "vault").unwrap().next.as_deref(), Some("safe"));

        get_key(&mut p, &g);
        assert!(p.is_ready(safe));
        // With the key, every option is there to choose from, not just the plain one.
        assert_eq!(status(&p, &g, "vault", "safe", "gold"), ChoiceStatus::Open);
        assert_eq!(status(&p, &g, "vault", "safe", "map"), ChoiceStatus::Open);
        let wise = p.choose(&g, "vault", "safe", "map").unwrap();
        assert_eq!(wise.next.as_deref(), Some("wise"));
        assert_eq!(wise.found.clues, vec!["map".to_string()]);
        assert_eq!(status(&p, &g, "vault", "safe", "gold"), ChoiceStatus::Closed);
        let after_decision = holdings(&p);

        // The story ended; a new scan restarts it for reading, decision kept.
        assert_eq!(p.scan(&g, "vault").unwrap().next.as_deref(), Some("safe"));
        assert!(p.choose(&g, "vault", "safe", "gold").is_none());
        assert!(p.choose(&g, "vault", "safe", "map").unwrap().found.is_empty());
        assert_eq!(holdings(&p), after_decision);
    }

    #[test]
    fn options_never_available_stay_hidden_after_the_decision() {
        let g = Game::from_toml(
            &include_str!("fixture.toml").replace("requires = { items = { key = 1 } }\n", ""),
        )
        .unwrap();
        let mut p = Progress::default();
        p.scan(&g, "vault");
        assert_eq!(status(&p, &g, "vault", "safe", "map"), ChoiceStatus::Hidden);
        p.choose(&g, "vault", "safe", "gold").unwrap();
        assert_eq!(status(&p, &g, "vault", "safe", "map"), ChoiceStatus::Hidden);
        // Getting the key later doesn't reopen the decision.
        get_key(&mut p, &g);
        assert_eq!(status(&p, &g, "vault", "safe", "map"), ChoiceStatus::Closed);
        assert!(p.choose(&g, "vault", "safe", "map").is_none());
    }

    /// Plays thousands of random steps, including ones the screen would never
    /// offer (options of scenes not reached, other options of decisions
    /// already made, passwords guessed anywhere), and checks after each one
    /// that no final decision has two options taken and that no two
    /// mutually exclusive outcomes were ever both reached.
    #[test]
    fn random_play_never_takes_both_sides_of_a_decision() {
        let fixture = game();
        let galaxy = Game::from_toml(include_str!("../../games/galaxy/game.toml")).unwrap();
        // Pairs of things a player must never have both of.
        let exclusive: [(&Game, &[(&str, &str)]); 2] = [
            (&fixture, &[("vault/rich", "vault/wise"), ("door/pay", "door/leave")]),
            (&galaxy, &[("flag/took_chart", "flag/left_chart")]),
        ];
        for (g, pairs) in exclusive {
            type Step<'g> = Box<dyn Fn(&mut Progress) + 'g>;
            let mut steps: Vec<Step> = Vec::new();
            for (lid, loc) in &g.locations {
                let lid = lid.clone();
                steps.push(Box::new({
                    let lid = lid.clone();
                    move |p| drop(p.scan(g, &lid))
                }));
                for (sid, scene) in &loc.scenes {
                    let (l, s) = (lid.clone(), sid.clone());
                    steps.push(Box::new(move |p| drop(p.continue_on(g, &l, &s))));
                    for c in &scene.choices {
                        let (l, s, c) = (lid.clone(), sid.clone(), c.id.clone());
                        steps.push(Box::new(move |p| drop(p.choose(g, &l, &s, &c))));
                    }
                    if let Some(pw) = &scene.password {
                        for guess in [pw.answers[0].clone(), "wrong".to_string()] {
                            let (l, s) = (lid.clone(), sid.clone());
                            steps.push(Box::new(move |p| drop(p.answer(g, &l, &s, &guess))));
                        }
                    }
                }
            }
            let has = |p: &Progress, key: &str| -> bool {
                let (kind, rest) = key.split_once('/').unwrap();
                match kind {
                    "flag" => p.flags.contains(rest),
                    _ => {
                        // "location/scene" reached, or "location/option" taken there.
                        let place = p.places.get(kind);
                        place.is_some_and(|pl| {
                            pl.seen.contains(rest) || pl.chosen.values().any(|c| c.contains(rest))
                        })
                    }
                }
            };
            let mut seed: u64 = 0x5eed;
            let mut decided = 0;
            for _game in 0..300 {
                let mut p = Progress::default();
                for _ in 0..60 {
                    // xorshift: a tiny repeatable random number generator.
                    seed ^= seed << 13;
                    seed ^= seed >> 7;
                    seed ^= seed << 17;
                    steps[(seed % steps.len() as u64) as usize](&mut p);
                    for (lid, place) in &p.places {
                        for (sid, chosen) in &place.chosen {
                            if g.scene(lid, sid).unwrap().pick == Pick::One {
                                assert!(chosen.len() <= 1, "{lid}/{sid}: two options taken: {chosen:?}");
                            }
                        }
                    }
                    for (a, b) in pairs.iter() {
                        assert!(!(has(&p, a) && has(&p, b)), "both {a} and {b}: {p:#?}");
                    }
                }
                decided += pairs.iter().any(|(a, b)| has(&p, a) || has(&p, b)) as u32;
            }
            // The walk must actually reach the decisions for the test to mean anything.
            assert!(decided > 100, "only {decided} of 300 random games reached a decision");
        }
    }

    #[test]
    fn scans_resume_or_restart() {
        let g = game();
        let mut p = Progress::default();
        reach_room(&mut p, &g);
        assert_eq!(p.scan(&g, "door").unwrap().next.as_deref(), Some("room"));
        p.choose(&g, "door", "room", "leave").unwrap();
        assert_eq!(p.scan(&g, "door").unwrap().next.as_deref(), Some("hall"));
        assert_eq!(p.visited, vec!["door".to_string(), "gate".to_string()]);
    }

    #[test]
    fn sources_record_where_things_came_from() {
        let g = game();
        let mut p = Progress::default();
        p.scan(&g, "door");
        assert_eq!(p.where_item_found(&g, "coin").len(), 1);
        assert!(p.where_item_found(&g, "key").is_empty());
    }

    #[test]
    fn unknown_ids_are_dropped() {
        let g = game();
        let mut p = Progress::default();
        p.scan(&g, "door");
        p.items.insert("removed_item".into(), 3);
        p.places.insert("removed_place".into(), Place::default());
        p.places.get_mut("door").unwrap().seen.insert("removed_scene".into());
        p.retain_known(&g);
        assert!(!p.items.contains_key("removed_item"));
        assert!(!p.places.contains_key("removed_place"));
        assert!(!p.has_seen("door", "removed_scene"));
    }
}
