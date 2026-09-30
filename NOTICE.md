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

### Status: AGPL-3.0 chosen for the repository (2026-09-28)

The owner has decided to publish this work open source, so option (a) applies:
the repository is AGPL-3.0-or-later, and `LICENSE` carries the full text. That
settles the source-publication question, including AGPL section 13 for any
hosted instance: the source is public.

### It does not settle the App Store

Publishing the source satisfies the AGPL's *source* obligation. It does not
satisfy its *no further restrictions* obligation when an app is distributed
through Apple's App Store or TestFlight.

The App Store terms impose conditions on what a recipient may do with the
software - device limits, no redistribution, DRM. AGPL section 10 forbids the
distributor from imposing any further restrictions on the rights it grants.
A distributor who is the sole copyright holder can waive this for their own
code, but Swiss Ephemeris is Astrodienst's, and only they can. This is why
GPL-licensed apps have been removed from the App Store before, VLC being the
well-known case.

So, for an iOS build specifically, three ways forward:

1. **Buy the Swiss Ephemeris Professional Licence** from Astrodienst. Cleanest
   and quickest: nothing in the code changes, your own code can stay AGPL, and
   the App Store conflict disappears because the ephemeris is no longer under
   AGPL terms.
2. **Ship only as a web app or PWA.** AGPL is entirely comfortable there: no
   Apple terms are involved, and publishing the source satisfies section 13.
   Costs nothing, but there is no App Store presence.
3. **Replace Swiss Ephemeris** with a permissively licensed or public-domain
   ephemeris. The architecture already isolates it: only `crates/swe-sys` and
   `crates/lagn-ephem` touch it, and everything above depends on
   `lagn-ephem`'s interface rather than on Swiss Ephemeris. The work is real -
   sidereal modes, ayanamsa handling and the validation that proved
   0.001 arcsec agreement - and it is the part of the system where subtle
   errors are hardest to detect. `swetest` could still be used as a test-only
   oracle, since testing is not distribution.

Option 1 is the recommendation if the App Store matters. Option 3 is the only
one that makes the project entirely free of a commercial dependency, and the
layering was built to keep it possible.
