# 0005: Make the game a room-based action game, not a survivor game

- Status: decided
- Date: 2026-11-20
- Issue: none. The maintainer decided this directly, without a Proposal issue, from the ideas in #41.

## Context

The prototype was built as a survivor game: weapons fire on their own, hundreds of enemies walk at the player, and experience gems give level-up picks in one endless arena. The maintainer wants it to lean towards action games such as Hades, Wizard of Legend and Hyper Light Drifter instead, with a start-of-run pick of skills, one shared pool of upgrades that mostly add elements to skills, and monsters whose move patterns draw on Monster Hunter: World's Deviljho and Alatreon (#41). That changes almost every system in the core, so the pillars had to be fixed before the plan in `ROADMAP.md` could be written.

## Decision

- **Genre.** A room-based action game: a run is a chain of arena rooms with a card pick after each clear, ending in a boss. No hordes; about 10 enemies at once at most.
- **The player's actions.** The player aims with the mouse or the right stick, controls three active skills, and has a roll as the one fixed defensive move. There is no dash action; some skills move the player like a dash.
- **Skills are balanced by timing, not cooldowns.** Light skills have short startup and recovery, heavy skills long ones. The player may pick any three skills, all heavy if they like.
- **One card pool.** The starting passive, the pick after each room and the skill upgrades come from the same pool. A card attaches to a skill or to the character, and most cards add an element to a skill. A skill carries one element at most.
- **The first elements** are fire, ice, lightning and celestial (gravity and stars).
- **Fair warnings.** Every enemy attack is warned on the ground before it lands, with a minimum warning time.
- **Bosses are original.** They borrow move ideas from Deviljho and Alatreon (a relentless, enraging chaser; an element-switching boss with an element check), not their names or looks.

The details, such as skill lists, timings and boss moves, live in `ROADMAP.md` and can change without a new record.

## Reasons

The maintainer chose the direction in #41. Balancing skills by startup and recovery rather than cooldowns keeps every button usable all the time and makes the choice between light and heavy skills about risk and timing, which suits bosses built around reading attacks. A single card pool keeps the start-of-run pick, the room rewards and the upgrades as one system with one card type, and it carries #29's two axes (a martial art plus a magic element) into the new shape. A roll without a cooldown, and dash skills without invulnerability, keep defence in one place.

Set aside: keeping the survivor game and adding action controls on top (auto weapons and hordes work against reading individual attacks), cooldown-based skill slots in the style of Wizard of Legend, and keeping a separate dash action next to the roll.

## Consequences

- The auto weapons, contact damage, experience gems and the 500-enemy cap are removed; the old roadmap's step for scaling to thousands of enemies no longer applies.
- The core needs a fixed time step, an input format with aim and buttons, timed hitboxes, enemy and boss state machines and data-driven content; see `ROADMAP.md`.
- #28, which pinned the current weapons in tests, and #29, the martial art and element proposal, are replaced by this direction.
- Touch controls need their own design before they are built, because aim plus three skills and a roll do not map to a phone easily.
- Changing the genre, the timing-based balance, the single card pool or the one fixed defensive move needs a new record that supersedes this one.
