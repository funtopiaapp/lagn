import Capacitor
import Foundation

/// The iOS half of the bridge: the engine compiled into the app, reached
/// through the C ABI in `include/lagn.h`.
///
/// Every method takes and returns a JSON string, so the web layer sees the
/// same shapes it gets from the HTTP API. Nothing here interprets a chart; it
/// moves strings and manages their lifetime.
///
/// Add to the app target together with `liblagn_ffi.a` and a bridging header
/// containing `#import "lagn.h"`. See `docs/phase11/APPLE.md`.
@objc(LagnNativePlugin)
public class LagnNativePlugin: CAPPlugin, CAPBridgedPlugin {
    public let identifier = "LagnNativePlugin"
    public let jsName = "LagnNative"
    public let pluginMethods: [CAPPluginMethod] = [
        CAPPluginMethod(name: "prepare", returnType: CAPPluginReturnPromise),
        CAPPluginMethod(name: "version", returnType: CAPPluginReturnPromise),
        CAPPluginMethod(name: "topics", returnType: CAPPluginReturnPromise),
        CAPPluginMethod(name: "chart", returnType: CAPPluginReturnPromise),
        CAPPluginMethod(name: "topic", returnType: CAPPluginReturnPromise),
        CAPPluginMethod(name: "periods", returnType: CAPPluginReturnPromise),
        CAPPluginMethod(name: "family", returnType: CAPPluginReturnPromise),
        CAPPluginMethod(name: "match", returnType: CAPPluginReturnPromise),
        CAPPluginMethod(name: "places", returnType: CAPPluginReturnPromise),
        CAPPluginMethod(name: "offset", returnType: CAPPluginReturnPromise),
    ]

    /// Readings run off the main thread: the heaviest (decades of transits)
    /// takes a few hundred milliseconds, which would visibly stall the UI.
    private let queue = DispatchQueue(label: "app.lagn.engine", qos: .userInitiated)

    /// Take ownership of a string from the engine and release it.
    private func take(_ pointer: UnsafeMutablePointer<CChar>?) -> String? {
        guard let pointer else { return nil }
        defer { lagn_string_free(pointer) }
        return String(cString: pointer)
    }

    /// Load the reference data once, whenever it is first needed. Making this
    /// lazy removes an ordering dependency: no call can arrive "too early".
    private static let ready: String = {
        do {
            let root = try prepareData()
            let config: [String: String] = [
                "ephemeris": root.appendingPathComponent("ephe").path,
                "corpus": root.appendingPathComponent("corpus").path,
                "places": root.appendingPathComponent("data/places.tsv").path,
                "tzdb": root.appendingPathComponent("data/zoneinfo").path,
            ]
            let json = String(data: try JSONSerialization.data(withJSONObject: config), encoding: .utf8)!
            guard let result = lagn_init(json) else { return #"{"error":"out of memory"}"# }
            defer { lagn_string_free(result) }
            return String(cString: result)
        } catch {
            return #"{"error":"could not prepare the reference data: \(error.localizedDescription)"}"#
        }
    }()

    /// Run `work` off the main thread and resolve with `{ value: <json> }`.
    private func answer(_ call: CAPPluginCall, _ work: @escaping () -> String?) {
        queue.async {
            // Touching `ready` loads the data on the first call and never again.
            let loaded = Self.ready
            if loaded.contains("\"error\"") {
                call.resolve(["value": loaded])
                return
            }
            guard let json = work() else {
                call.reject("The engine ran out of memory.")
                return
            }
            call.resolve(["value": json])
        }
    }

    /// A required string argument, or a rejected call.
    private func string(_ call: CAPPluginCall, _ name: String) -> String? {
        guard let value = call.getString(name) else {
            call.reject("\(name) is required")
            return nil
        }
        return value
    }

    // MARK: - Lifecycle

    /// Warm the engine up at launch, so the first reading is not the call
    /// that pays the loading cost. Optional: every method loads it anyway.
    @objc func prepare(_ call: CAPPluginCall) {
        answer(call) { Self.ready }
    }

    /// Where the data lives on the device, copied out of the bundle once.
    /// Application Support is excluded from iCloud backup here: it is 12 MB of
    /// reference data that ships with every build.
    static func prepareData() throws -> URL {
        let fm = FileManager.default
        var root = try fm.url(for: .applicationSupportDirectory, in: .userDomainMask, appropriateFor: nil, create: true)
            .appendingPathComponent("lagn-data", isDirectory: true)

        guard let bundled = Bundle.main.url(forResource: "data", withExtension: nil) else {
            throw NSError(domain: "app.lagn", code: 1,
                          userInfo: [NSLocalizedDescriptionKey: "the data folder is missing from the app bundle"])
        }

        // Re-copy when the app has been updated, so a new corpus ships with it.
        let stamp = root.appendingPathComponent(".version")
        let build = Bundle.main.infoDictionary?["CFBundleVersion"] as? String ?? "0"
        let current = try? String(contentsOf: stamp, encoding: .utf8)
        if current != build {
            if fm.fileExists(atPath: root.path) {
                try fm.removeItem(at: root)
            }
            try fm.copyItem(at: bundled, to: root)
            try build.write(to: stamp, atomically: true, encoding: .utf8)
            var values = URLResourceValues()
            values.isExcludedFromBackup = true
            try? root.setResourceValues(&values)
        }
        return root
    }

    // MARK: - Readings

    @objc func version(_ call: CAPPluginCall) {
        answer(call) { self.take(lagn_version()) }
    }

    @objc func topics(_ call: CAPPluginCall) {
        answer(call) { self.take(lagn_topics()) }
    }

    @objc func chart(_ call: CAPPluginCall) {
        guard let request = string(call, "request") else { return }
        answer(call) { self.take(lagn_chart(request)) }
    }

    @objc func topic(_ call: CAPPluginCall) {
        guard let name = string(call, "name"), let request = string(call, "request") else { return }
        answer(call) { self.take(lagn_topic(name, request)) }
    }

    @objc func periods(_ call: CAPPluginCall) {
        guard let request = string(call, "request") else { return }
        answer(call) { self.take(lagn_periods(request)) }
    }

    @objc func family(_ call: CAPPluginCall) {
        guard let request = string(call, "request") else { return }
        answer(call) { self.take(lagn_family(request)) }
    }

    @objc func match(_ call: CAPPluginCall) {
        guard let request = string(call, "request") else { return }
        answer(call) { self.take(lagn_match(request)) }
    }

    @objc func places(_ call: CAPPluginCall) {
        guard let request = string(call, "request") else { return }
        answer(call) { self.take(lagn_places(request)) }
    }

    @objc func offset(_ call: CAPPluginCall) {
        guard let request = string(call, "request") else { return }
        answer(call) { self.take(lagn_offset(request)) }
    }
}
