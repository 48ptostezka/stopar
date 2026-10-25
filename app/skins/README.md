# Skins

A skin is the whole look of the app: colours, fonts, the badge shape on the
menu keys, the side-rail art and the small marker used on the rails and in
page headers. Screens and behaviour never depend on the skin, and
`app/style/frame.css` never names a colour or a font.

The game file picks its skin (`skin = "orbit"` in `[game]`). For a quick
preview of another one, override it at build time:
`STOPA_SKIN=trail trunk serve`.

## Files

| File | Required | What it is |
| --- | --- | --- |
| `skin.css` | yes | Tokens on `:root`, `@font-face` rules, and the few selectors below |
| `markers.svg` | yes | One `<symbol id="marker">`, drawn in `currentColor` |
| `fonts/*.woff2` | no | Fonts the skin uses. Must cover Czech (ě š č ř ž ý á í é ů ú ň ť ď) |
| anything else | no | Rail tiles and other art referenced from `skin.css` |

The app loads `skins/<skin>/skin.css` at startup and shows nothing until it
has loaded, so there is no unstyled flash.

## Tokens

Set these on `:root`. Anything left out keeps the default from `frame.css`.

| Token | Meaning |
| --- | --- |
| `--theme-color` | Phone status bar colour |
| `--panel` | Background of the central panel |
| `--surface` | Keys, cards, tables |
| `--ink` | Text, borders, shadows |
| `--muted` | Secondary text |
| `--line` | Quiet dividers |
| `--on-accent`, `--on-ink` | Text on an accent colour, text on ink |
| `--accent-1` … `--accent-4` | Page colours. Menu entries pick one each (`app/src/menu.rs`) |
| `--font-display`, `--font-body` | Font stacks for labels and headings, and for body text |
| `--display-weight`, `--display-transform`, `--display-tracking` | Weight, letter case (`uppercase`, `none`) and letter spacing of labels and headings |
| `--radius`, `--radius-small` | Corner radius of large and small keys |
| `--border` | Border width |
| `--depth-x`, `--depth-y` | How far a key sits above its shadow; pressing moves it by this much |
| `--key-shadow`, `--key-shadow-small` | Override for shadows that aren't one solid offset (Case file stacks a second sheet) |
| `--size-key-label`, `--size-heading`, `--size-title` | Type sizes of menu labels, page titles and the game title |
| `--rail-edge` | Width of the ink line where rails meet the panel |
| `--marker-size`, `--marker-gap` | Size and vertical spacing of rail markers |
| `--badge-size`, `--badge-inset`, `--badge-gap` | Badge size, its distance from the key's left edge, gap before the label |

At runtime the frame also sets `--key-accent` on each menu key and
`--page-accent` on each page; a skin reads them, never sets them.

## Selectors a skin may style

- `.rail` — the rail background. Put `url()` art here, in a normal rule.
- `.badge` — the badge shape. Fill with `var(--key-accent)`.
- `.menu::before` — a decoration behind the menu keys (Trail's signpost pole).
  Use `z-index: -1`; the panel keeps it above its own background.
- `.tag`, `.key--nav:nth-child(…)` — small touches, such as Case file's
  tilted labels.

Keep everything else to tokens, so a new screen looks right in every skin
without touching the skins.

## The marker

`markers.svg` holds one symbol, `<symbol id="marker" viewBox="-12 -12 24 24">`,
used on the rails and at the right of page headers. Fill the coloured part
with `currentColor`; the app sets it to the page or menu colour. Two variables
reach inside the symbol through `style="stroke: var(…)"`:

- `--marker-outline`: unset on the rails, `var(--ink)` in page headers.
- `--marker-ring`: the same, for ring-like details (Orbit's planet ring).

## Gotchas

- Write `url()` in normal rules, not inside custom properties. Browsers
  resolve relative URLs in custom properties inconsistently.
- XML comments can't contain two hyphens in a row. One stray `--` in a
  comment in `markers.svg` makes the whole file invalid and the markers vanish.
- Test with long Czech labels (`Nastavení`, `Inventář`) at 320 px wide.

## Adding a skin

1. Copy the folder closest to what you want, e.g. `cp -r orbit reef`.
2. Change the tokens, badge, rail art and marker. Swap the fonts if needed
   (subset them to Latin + Latin Extended-A, woff2).
3. Preview with `STOPA_SKIN=reef trunk serve`, then set `skin = "reef"` in
   the game file.

## Included skins

| Skin | Idea | Fonts |
| --- | --- | --- |
| `orbit` | Light console panel between starry rails, planet badges | Saira, Saira Wide (OFL) |
| `trail` | Czech hiking-trail signposts: blaze badges, pole, spruce rails | Barlow, Barlow Condensed (OFL) |
| `case` | Detective's desk: manila, stacked paper keys, evidence labels, cork | Special Elite (Apache 2.0), Barlow (OFL) |
