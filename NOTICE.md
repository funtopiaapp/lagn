# Third-party notices

## Swiss Ephemeris

`crates/swe-sys/vendor/` and `ephe/` contain Swiss Ephemeris, Copyright (C)
1997-2021 Astrodienst AG, Switzerland. Full terms in
`crates/swe-sys/vendor/LICENSE-swisseph.txt`.

Swiss Ephemeris is **dual-licensed**. A developer must choose one *before*
distributing software containing it, and before activating any public service
that uses it:

**a) GNU Affero General Public License v3.** Requires publishing the complete
source of the combined work - including, for a hosted web app, source served
to users over a network. For this project that would mean publishing the rule
corpus, which is the product.

**b) Swiss Ephemeris Professional License.** A paid commercial licence from
Astrodienst that removes the source-publication requirement.

### Status: UNRESOLVED - decide before any deployment

This repository currently declares AGPL-3.0-or-later, which is the safe
default for private development. That choice is **not** compatible with
shipping a closed-source web, iOS or Android app.

Resolve this before Phase 4. Contact Astrodienst (https://www.astro.com/swisseph/)
for current Professional Licence terms; it has historically been a one-time
fee, but confirm directly rather than relying on any figure quoted elsewhere.

If the commercial licence is not acquired, the alternatives are to publish
everything under AGPL, or to replace Swiss Ephemeris with a permissively
licensed ephemeris - which means reimplementing sidereal modes and ayanamsa
handling, the part of the system where subtle errors are hardest to detect.
