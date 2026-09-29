/* A plain C program that uses the engine exactly as an iOS app will: link the
 * library, call lagn_init once, then ask for readings. No server, no network.
 *
 *   bash scripts/native-demo.sh
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include "lagn.h"

/* Print the first `width` characters of a JSON value, for a readable demo. */
static void show(const char *label, const char *json, size_t width) {
    printf("  %-22s %.*s%s\n", label, (int)width, json, strlen(json) > width ? "..." : "");
}

/* Pull one string value out of flat JSON. Enough for a demo; the real client
 * uses a JSON parser. */
static void field(const char *json, const char *key, char *out, size_t n) {
    char pattern[64];
    snprintf(pattern, sizeof pattern, "\"%s\":\"", key);
    const char *p = strstr(json, pattern);
    out[0] = '\0';
    if (!p) return;
    p += strlen(pattern);
    const char *end = strchr(p, '"');
    if (!end || (size_t)(end - p) >= n) return;
    memcpy(out, p, end - p);
    out[end - p] = '\0';
}

static double seconds_since(struct timespec start) {
    struct timespec now;
    clock_gettime(CLOCK_MONOTONIC, &now);
    return (now.tv_sec - start.tv_sec) + (now.tv_nsec - start.tv_nsec) / 1e9;
}

int main(int argc, char **argv) {
    const char *root = argc > 1 ? argv[1] : ".";
    char config[2048];
    snprintf(config, sizeof config,
             "{\"ephemeris\":\"%s/ephe\",\"corpus\":\"%s/corpus\","
             "\"places\":\"%s/data/places.tsv\",\"tzdb\":\"%s/data/zoneinfo\"}",
             root, root, root, root);

    printf("\nlagn, running in a C program with no server\n\n");

    struct timespec t0;
    clock_gettime(CLOCK_MONOTONIC, &t0);
    char *init = lagn_init(config);
    if (!init || strstr(init, "\"error\"")) {
        fprintf(stderr, "init failed: %s\n", init ? init : "(null)");
        lagn_string_free(init);
        return 1;
    }
    printf("  loaded in %.2f s, ready = %s\n\n", seconds_since(t0), lagn_ready() ? "yes" : "no");
    lagn_string_free(init);

    char *version = lagn_version();
    char engine[64], swiss[64], tzdb[64];
    field(version, "engine", engine, sizeof engine);
    field(version, "swiss_ephemeris", swiss, sizeof swiss);
    field(version, "tzdb", tzdb, sizeof tzdb);
    printf("  engine %s, Swiss Ephemeris %s, tz database %s\n", engine, swiss, tzdb);
    lagn_string_free(version);

    /* A birth: 21 June 1985, 14:30, Chennai. */
    const char *birth =
        "{\"date\":\"1985-06-21\",\"time\":\"14:30:00\","
        "\"latitude\":13.08,\"longitude\":80.27,\"utc_offset_hours\":5.5}";
    char request[1024];
    snprintf(request, sizeof request, "{\"birth\":%s}", birth);

    printf("\n  chart\n");
    clock_gettime(CLOCK_MONOTONIC, &t0);
    char *chart = lagn_chart(request);
    double chart_s = seconds_since(t0);
    char lagna[64];
    field(chart, "rasi", lagna, sizeof lagna);
    printf("    lagna %s, computed in %.0f ms\n", lagna, chart_s * 1000);
    lagn_string_free(chart);

    printf("\n  readings\n");
    char *topics = lagn_topics();
    /* Walk the topic ids and read each one. */
    const char *p = topics;
    int count = 0;
    double total = 0;
    while ((p = strstr(p, "\"id\":\"")) != NULL) {
        p += 6;
        const char *end = strchr(p, '"');
        char id[64];
        size_t n = (size_t)(end - p);
        if (n >= sizeof id) break;
        memcpy(id, p, n);
        id[n] = '\0';

        clock_gettime(CLOCK_MONOTONIC, &t0);
        char *reading = lagn_topic(id, request);
        total += seconds_since(t0);
        if (strstr(reading, "\"error\"")) {
            printf("    %-12s %s\n", id, reading);
        } else {
            /* The first line of the write-up is the conclusion. */
            const char *summary = strstr(reading, "\"summary\":[\"");
            char text[220] = "";
            if (summary) {
                summary += strlen("\"summary\":[\"");
                const char *e = strchr(summary, '"');
                size_t len = e && (size_t)(e - summary) < sizeof text ? (size_t)(e - summary) : sizeof text - 1;
                memcpy(text, summary, len);
                text[len] = '\0';
            }
            printf("    %-12s %.*s%s\n", id, 150, text, strlen(text) > 150 ? "..." : "");
        }
        lagn_string_free(reading);
        count++;
    }
    lagn_string_free(topics);
    printf("\n    %d readings in %.0f ms\n", count, total * 1000);

    printf("\n  sensitive periods\n");
    snprintf(request, sizeof request, "{\"birth\":%s,\"from_age\":20,\"to_age\":45}", birth);
    clock_gettime(CLOCK_MONOTONIC, &t0);
    char *periods = lagn_periods(request);
    double periods_s = seconds_since(t0);
    int windows = 0;
    for (const char *w = periods; (w = strstr(w, "\"maha\":")) != NULL; w += 7) windows++;
    printf("    %d dasha windows with transits, in %.0f ms\n", windows, periods_s * 1000);
    show("first window", periods, 90);
    lagn_string_free(periods);

    printf("\n  place search\n");
    char *places = lagn_places("{\"q\":\"Chennai\",\"limit\":3}");
    show("Chennai", places, 100);
    lagn_string_free(places);

    printf("\n  errors are data, not crashes\n");
    /* Each of these fails at a different stage, and each says which. */
    struct { const char *label; const char *request; } bad[] = {
        {"malformed JSON",  "{\"birth\": {"},
        {"missing field",   "{\"birth\":{\"date\":\"1985-06-21\"}}"},
        {"bad date",        "{\"birth\":{\"date\":\"not-a-date\",\"time\":\"14:30:00\","
                            "\"latitude\":13.08,\"longitude\":80.27,\"utc_offset_hours\":5.5}}"},
        {"impossible date", "{\"birth\":{\"date\":\"1985-02-30\",\"time\":\"14:30:00\","
                            "\"latitude\":13.08,\"longitude\":80.27,\"utc_offset_hours\":5.5}}"},
        {"bad time",        "{\"birth\":{\"date\":\"1985-06-21\",\"time\":\"25:00:00\","
                            "\"latitude\":13.08,\"longitude\":80.27,\"utc_offset_hours\":5.5}}"},
        {"latitude off Earth", "{\"birth\":{\"date\":\"1985-06-21\",\"time\":\"14:30:00\","
                            "\"latitude\":999,\"longitude\":80.27,\"utc_offset_hours\":5.5}}"},
    };
    for (size_t i = 0; i < sizeof bad / sizeof bad[0]; i++) {
        char *answer = lagn_chart(bad[i].request);
        show(bad[i].label, answer, 110);
        lagn_string_free(answer);
    }
    char *null_arg = lagn_chart(NULL);
    show("null pointer", null_arg, 110);
    lagn_string_free(null_arg);
    char *unknown = lagn_topic("no-such-topic", request);
    show("unknown topic", unknown, 110);
    lagn_string_free(unknown);

    printf("\n");
    return 0;
}
