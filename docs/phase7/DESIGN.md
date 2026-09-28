# Phase 7 design: interface

Status: approved for build (product owner chose "modern app" and
"verdict first, then tabs" on 2026-09-23).

## 1. Why

Two pieces of product owner feedback:

1. The interface was not liked, and light/dark had to follow the operating
   system with no way to choose.
2. In the family view, the relationship between the native's reading and the
   member's own reading was not clear.

Nothing about the engine changes. This is the shell, the theme and the family
presentation, plus one new piece of generated text (section 4).

## 2. Theme (light and dark, chosen)

Three settings: **System** (default), **Light**, **Dark**, in a segmented
control in the header. The choice is kept in `localStorage` under
`lagn.theme`; storage may be unavailable, so every access is guarded and the
app still works for the page view.

- Tokens live on `:root` for light; `:root[data-theme="dark"]` overrides them;
  `@media (prefers-color-scheme: dark)` overrides them too, but only under
  `:root:not([data-theme="light"])`, so an explicit choice always wins.
- With "System", the app follows later OS changes live (`matchMedia` listener).
- `<meta name="theme-color">` is updated to the surface colour so the browser
  and the wrapped app chrome match.
- Contrast: body text at least 4.5:1 against its background, and large text,
  borders and chips at least 3:1, in **both** themes. A browser test measures
  this on the real rendered page rather than trusting the palette.

## 3. Shell ("modern app")

- **Sticky header:** app name, theme control, and "New chart" once a chart
  exists.
- **Segmented tabs** under it, sticky, scrollable sideways on a phone, with the
  current tab marked by `aria-current`.
- **Cards** with a light border and a small shadow; a card header carries the
  title and a status chip.
- **Chips** state the verdict at a glance: the score, "supportive" or "calls
  for care", "sensitive", "now", "draft". Colour is never the only carrier:
  every chip has text.
- **Findings** are colour-coded down the left edge (supportive, care, noted),
  keeping the same wording and structure as today.
- Touch targets stay at least 44 px; inputs stay at least 16 px to stop iOS
  zooming; no horizontal page scroll at phone widths.

## 4. Family: verdict first, then tabs

Per member:

1. **Verdict card:** name and relation; one plain sentence on whether the two
   readings agree; and the two leans side by side, each labelled with whose
   chart it is and what it was read for.
2. **Tabs:** *Your chart* (the native's chart read for that relation),
   *<Name>'s chart* (their own topics), *Together*.
3. **Together** is new text from the engine (`lagn_rules::family::comparison`,
   deterministic templates, no model): why two charts are read, what each one
   says in plain words, what agreement or disagreement means, and what to do
   with it. It never averages the two, and never overrides one with the other.

Names are used rather than pronouns, since a member's pronouns are not known.

## 5. Acceptance

1. Theme: choosing Light or Dark overrides the OS setting, survives a reload,
   and updates `theme-color`; "System" follows the OS live. Unit-tested, and
   a browser test switches and reloads.
2. Contrast measured on the rendered page passes in both themes.
3. The family view shows the verdict, then the three tabs, and the *Together*
   text equals the engine's own output.
4. `comparison` is deterministic, passes the banned-wording and tone scans of
   phase 6, and states both readings without merging them.
5. Every phase 4-6 browser and unit test still passes, or is updated where the
   markup deliberately changed; phone ergonomics unchanged.
