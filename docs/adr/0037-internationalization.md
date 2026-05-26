# ADR-0037: Internationalization (i18n)

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

Almost every user-visible string in the game (menu labels, button
captions, HUD legends, error messages, tool tips) must be
translatable. The choice of i18n approach has long-lasting
consequences:

- A **flat key-value translation system** (Gettext, ICU MessageFormat,
  Fluent, ...) keeps lookups dynamic, supports rich grammar features
  (plurals, gendered forms, complex substitutions), and is the
  industry standard. Missing translations are detected only at
  runtime.
- A **statically typed, compile-time-verified** approach (translation
  files compiled into a Rust struct hierarchy) catches missing or
  mistyped keys at compile time, at the cost of less flexible
  runtime grammar.

The project's near-term needs are short, fixed strings: menu items,
labels, units. Long, grammatically rich strings ("the trader offers
{} units. Price tiers are: ...") are not yet on the roadmap. Fluent
and similar runtime systems would solve a problem we do not yet
have, while introducing a class of error (missing key at runtime)
that we explicitly want to avoid (per the spirit of
[ADR-0013](0013-no-silent-fallbacks.md)).

## Decision

**For the foreseeable future, translations are JSON files with a
fixed nested structure that mirrors a typed Rust hierarchy.** Keys
are validated at **compile time**; missing or mistyped keys are
compiler errors, not runtime warnings.

### Storage

Translation files live at:

```
assets/i18n/<language>.json
```

with `<language>` an ISO 639-1 (or 639-3 where needed) language
code, optionally with a region suffix (`en`, `de`, `en-US`,
`pt-BR`). Initially: `en.json` (default), `de.json`.

The English file is the **reference**: it defines the complete set
of keys. Other languages may be incomplete; missing keys fall back
to the English value. This fallback is **not** a silent default in
the sense of [ADR-0013](0013-no-silent-fallbacks.md): the language
file omits a key by design (the translator has not got to it yet),
and the user has explicitly chosen "best effort translation".

A missing key in `en.json`, on the other hand, is an error: the
reference file must be complete.

### Structure

The file is a nested JSON object. Keys are `snake_case`. Leaves are
strings. Example:

```json
{
  "ui": {
    "menu": {
      "help": "Help (F1)",
      "quit": "Quit"
    },
    "hud": {
      "speed_label": "Speed",
      "speed_unit": "m/s"
    }
  }
}
```

There is **one** schema, `i18n.schema.json`, that defines this
shape. Adding a new key means editing the schema, the English
reference file and the Rust struct in lockstep.

### Access from Rust

Translation lookup goes through a typed struct hierarchy:

```rust
let help_text: &str = &i18n.ui.menu.help;
let speed_label: &str = &i18n.ui.hud.speed_label;
```

A typo (`i18n.ui.menue.help`) is a compile error. A missing key is a
compile error. There is no `.get("ui.menu.help")` or
`tr!("ui-menu-help")` form; both defeat the purpose of compile-time
checking.

The struct is hand-written and lives in `delta-v-core` (or a
dedicated `delta-v-i18n` crate if it grows). When a new key is
added, the struct is updated together with the schema and the
English file; CI fails until the three agree.

### Simple substitutions

For values like `"{} m/s"`, simple positional formatting via
`format!` or a thin helper is used:

```rust
let s = i18n_format!(i18n.ui.hud.speed_value, speed); // "120 m/s"
```

The helper is a wrapper around `format!` with a fixed argument
count per key (encoded in the schema as an `arg_count` annotation
or similar; deferred until needed).

We **do not** adopt:

- Fluent / ICU MessageFormat **as a replacement** for the typed
  struct approach. When a future feature genuinely needs
  grammatically rich strings (pluralisation, gendered forms,
  complex argument trees -- think a trading dialog with sentences
  like "the trader offers {n} units. Price tiers are: ..."), we
  introduce Fluent (or a similar runtime library) **alongside**
  the typed struct, scoped to that feature. We do **not** rewrite
  the existing UI strings.

  In that hybrid setup:
  - UI labels, button captions, menu entries, HUD legends and
    other short, fixed strings stay in the typed Rust hierarchy.
    Their main benefit -- typos and missing keys caught at
    compile time -- is preserved.
  - The new feature's grammatically rich strings live in Fluent
    files (or equivalent), loaded at runtime, with the runtime
    cost (missing-key checks at runtime, separate workflow
    discipline) localised to that feature.
  - A successor ADR documents the boundary precisely when the
    first such feature is built.
- Runtime translation-key strings (`tr!("ui.menu.help")`) for the
  typed-struct part. They push errors from compile time to
  runtime and defeat the main benefit of this ADR.
- Gettext / `.po` files. They are powerful but optimise for a
  workflow (translator tools, plural rules) we do not yet need.

### Language selection

The default language is `en`. The user can override the language
through the configuration system
([ADR-0010](0010-configuration-system.md)). Switching language at
runtime is supported in `--features dev` (via hot-reload, per
[ADR-0035](0035-hot-reload-of-configs.md)); in release builds it
takes effect on next start.

### What this is not

- Not a full localisation framework. We do not yet handle
  date/time formatting, currency, number formatting per locale,
  right-to-left layout, or font-set selection. Those become
  separate ADRs when they become relevant.
- Not a translator-tooling workflow (no `.po` / Crowdin / Weblate
  integration). Translators edit the JSON file for their language
  directly and submit a pull request; the maintainer (or
  CI tooling) keeps the reference English file, the schema and
  the typed Rust hierarchy in sync.

### Who produces translations

Translations are the **one form of external human contribution
that the project accepts** at this stage. The conditions are
documented in [ADR-0036](0036-external-human-contributors-process.md)
and the practical workflow lives in `CONTRIBUTING.md`. In short:

- A translator edits or adds a single file under `assets/i18n/`
  for their language.
- They do not touch Rust code or the schema; if a missing or
  unclear reference key blocks them, they open an issue.
- The PR is reviewed for content (does the translation read
  well?) and validated mechanically (does it pass the schema, and
  does every key it provides exist in the reference?).

The maintainer or AI tooling may produce a first-pass translation
for any new language; community translators then improve it.

## Consequences

Positive:

- Compile time catches a class of error that other i18n systems
  catch only at runtime.
- The cost of i18n for the current scope (short labels) is small:
  a struct, a schema, a JSON file per language.
- Migration to a richer system later is possible because the
  surface area (typed struct access) is small.

Negative:

- Adding a key is a three-place change (struct, schema, reference
  file). The CI compile-time check turns the discipline into
  mechanical feedback.
- Languages with strong grammar features (plurals, cases) cannot
  be served well by simple substitutions. Acceptable for now;
  revisited when needed.
- A new language is added by a developer (struct, schema,
  reference file all in Rust/JSON), not by a translator with a
  separate tool. Acceptable while the project is single-author.

Follow-up:

- The `i18n.schema.json` is added together with the first concrete
  translation work, likely during M6 (HUD) at the latest.
- When the first feature genuinely needs grammatically rich text
  (e.g. a trading dialog), a successor ADR documents how Fluent
  (or similar) is introduced **for that feature only**, alongside
  the typed struct approach which remains in place for all other
  strings.
