/* lagn: a deterministic Vedic astrology engine, callable from C, Swift or
 * Objective-C. Specification: docs/phase10/DESIGN.md.
 *
 * Every function returns a NUL-terminated UTF-8 JSON string that the caller
 * MUST release with lagn_string_free. NULL is returned only if the process is
 * out of memory. Failures come back as {"error": "..."}.
 *
 * Call lagn_init once before anything else. Every function is safe to call
 * from any thread.
 */
#ifndef LAGN_H
#define LAGN_H

#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Load the reference data. config_json:
 *   {"ephemeris": "<dir>", "corpus": "<dir>", "places": "<file>",
 *    "tzdb": "<dir>"}          tzdb is optional; a copy is compiled in.
 * Returns {"ok": true, "version": {...}} or {"error": "..."}. */
char *lagn_init(const char *config_json);

/* Whether lagn_init has succeeded. */
bool lagn_ready(void);

/* Engine, ephemeris, timezone and corpus versions. */
char *lagn_version(void);

/* The topic catalogue and the bhava meanings. */
char *lagn_topics(void);

/* {"birth": {...}} -> the chart, its vargas, dasha tree and ashtakavarga. */
char *lagn_chart(const char *request_json);

/* A reading for one topic; name is an id from lagn_topics. */
char *lagn_topic(const char *name, const char *request_json);

/* {"birth": {...}, "from_age": n, "to_age": n} -> sensitive periods. */
char *lagn_periods(const char *request_json);

/* A day's panchanga and its Rahu kalam, Yamagandam, Kuligai and Abhijit.
   Needs no chart: these timings belong to the day and the place. */
char *lagn_days(const char *request_json);

/* {"birth": {...}} -> chara karakas, arudha padas and argala.
   Professional surface only; the response carries the variant defaults
   that produced it. */
char *lagn_jaimini(const char *request_json);

/* {"birth": {...}} -> Chara dasha, the Jaimini rasi dasha.
   Professional surface only; carries its variant defaults. */
char *lagn_chara(const char *request_json);

/* {"birth": {...}} -> upagrahas (Gulika, Dhuma and the rest) and the
   Bhava, Hora and Ghati lagnas. Professional surface only. */
char *lagn_upagraha(const char *request_json);

/* {"birth": {...}} -> a Krishnamurti Paddhati reading: the four lords of
   every point, the Placidus cusps, the ruling planets and the house
   significators. Professional surface only. */
char *lagn_kp(const char *request_json);

/* {"native": {...}, "member": {...}, "relation": "..."} -> family reading. */
char *lagn_family(const char *request_json);

/* {"bride": {...}, "groom": {...}} -> the ten poruthams. */
char *lagn_match(const char *request_json);

/* {"q": "...", "limit": n} -> place search. */
char *lagn_places(const char *request_json);

/* {"tz": "...", "date": "...", "time": "...", "longitude": n} -> UTC offset. */
char *lagn_offset(const char *request_json);

/* Release a string returned by any function above. NULL is allowed. */
void lagn_string_free(char *p);

/* Allocate len bytes inside the module's memory, for hosts that cannot
 * otherwise place a string there (WebAssembly). Native callers pass their own
 * pointers and do not need these. */
unsigned char *lagn_buffer_alloc(size_t len);
void lagn_buffer_free(unsigned char *p, size_t len);

#ifdef __cplusplus
}
#endif
#endif /* LAGN_H */
