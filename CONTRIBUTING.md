# Working on lagn

## Branches

```
feature/<name> ──► develop ──► master
                      ▲            │
                      └── back-merge ┘
```

- **`master`** is the released line. Nothing is committed to it directly.
- **`develop`** carries release commits. Everything reaches `master` from here.
- **`feature/<name>`** is where a change is built, branched from `develop` and
  merged back into it. A feature reaches `master` only by way of `develop`.
- `master` is merged back into `develop` after each release, so the two do not
  drift.

```bash
git switch develop && git pull
git switch -c feature/my-change          # build it
git switch develop && git merge --no-ff feature/my-change

# release
git switch master && git merge --no-ff develop
git switch develop && git merge master   # back-merge
```

## Before a merge into develop

`make qa` must pass. It runs clippy, the whole test suite in release and debug,
the independent oracles (Phase 2 and 3, poruthams, transits, timezones), the
corpus validator, and the web unit and browser suites.

Before a release, run the same suite on Linux, which is the deployment
platform:

```bash
docker build --target qa -t lagn-qa . && docker run --rm lagn-qa
```

## Rules for changes to the corpus

Every rule carries a review (`reviewer`, `date`, reasoning), a plain-language
`text.impact`, and a provenance note. The validator and the test suite enforce
this, along with the wording guardrails in `docs/phase6/DESIGN.md` section 2
and `docs/phase9/DESIGN.md` section 2. A rule that fails those is not a style
problem; it is a content policy failure and the build fails.

The content is **AI-reviewed, not astrologer-reviewed**. Review sheets for a
practitioner are generated with `lagn review-sheet --topic <name>`.

## Licence

Unresolved, and it gates distribution. See `NOTICE.md`: Swiss Ephemeris is
AGPL-or-commercial, and the choice affects whether this repository and its rule
corpus can stay closed.
