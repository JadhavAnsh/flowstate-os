@preconcurrency import AVFAudio
@preconcurrency import AVFoundation
import Foundation
import FoundationModels
import Speech

private enum JSONValue: Codable, Sendable {
    case string(String)
    case bool(Bool)
    case number(Double)
    case array([JSONValue])
    case object([String: JSONValue])
    case null

    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        if container.decodeNil() { self = .null }
        else if let value = try? container.decode(Bool.self) { self = .bool(value) }
        else if let value = try? container.decode(Double.self) { self = .number(value) }
        else if let value = try? container.decode(String.self) { self = .string(value) }
        else if let value = try? container.decode([JSONValue].self) { self = .array(value) }
        else { self = .object(try container.decode([String: JSONValue].self)) }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        switch self {
        case .string(let value): try container.encode(value)
        case .bool(let value): try container.encode(value)
        case .number(let value): try container.encode(value)
        case .array(let value): try container.encode(value)
        case .object(let value): try container.encode(value)
        case .null: try container.encodeNil()
        }
    }
}

private struct CommandEnvelope: Decodable, Sendable {
    let id: String
    let command: String
    var locale: String?
    var conversationId: String?
    var prompt: String?
    var instructions: String?
    var text: String?
}

private struct EventEnvelope: Encodable, Sendable {
    let id: String
    let event: String
    var payload: [String: JSONValue] = [:]
    var code: String?
    var message: String?
}

private actor OutputWriter {
    private let encoder = JSONEncoder()

    func send(
        id: String,
        event: String,
        payload: [String: JSONValue] = [:],
        code: String? = nil,
        message: String? = nil
    ) {
        do {
            let envelope = EventEnvelope(
                id: id,
                event: event,
                payload: payload,
                code: code,
                message: message
            )
            var data = try encoder.encode(envelope)
            data.append(0x0A)
            FileHandle.standardOutput.write(data)
        } catch {
            FileHandle.standardError.write(Data("output error: \(error)\n".utf8))
        }
    }
}

private enum RuntimeError: LocalizedError {
    case unsupportedLocale(String)
    case speechAssetUnavailable(String)
    case microphoneDenied
    case captureAlreadyActive
    case captureNotActive
    case modelUnavailable(String)
    case invalidCommand(String)

    var errorDescription: String? {
        switch self {
        case .unsupportedLocale(let locale): "Speech transcription does not support \(locale)."
        case .speechAssetUnavailable(let locale): "Speech assets are unavailable for \(locale)."
        case .microphoneDenied: "Microphone access was denied."
        case .captureAlreadyActive: "Speech capture is already active."
        case .captureNotActive: "Speech capture is not active."
        case .modelUnavailable(let reason): "Apple Foundation Models is unavailable: \(reason)."
        case .invalidCommand(let command): "Unknown Apple runtime command: \(command)."
        }
    }
}

private func appleModelAvailability() -> (available: Bool, reason: String) {
    switch SystemLanguageModel.default.availability {
    case .available:
        return (true, "available")
    case .unavailable(.deviceNotEligible):
        return (false, "device_not_eligible")
    case .unavailable(.appleIntelligenceNotEnabled):
        return (false, "apple_intelligence_not_enabled")
    case .unavailable(.modelNotReady):
        return (false, "model_not_ready")
    case .unavailable:
        return (false, "unknown")
    }
}

private final class ConverterInput: @unchecked Sendable {
    private let lock = NSLock()
    private var supplied = false

    func take(_ buffer: AVAudioPCMBuffer) -> AVAudioPCMBuffer? {
        lock.withLock {
            guard !supplied else { return nil }
            supplied = true
            return buffer
        }
    }
}

private func preferredLocale(_ identifier: String?) async throws -> Locale {
    let requested = Locale(identifier: identifier ?? "en-IN")
    if let supported = await SpeechTranscriber.supportedLocale(equivalentTo: requested) {
        return supported
    }
    let fallback = Locale(identifier: "en-US")
    if let supported = await SpeechTranscriber.supportedLocale(equivalentTo: fallback) {
        return supported
    }
    throw RuntimeError.unsupportedLocale(requested.identifier)
}

private func transcriber(for locale: Locale) -> SpeechTranscriber {
    SpeechTranscriber(locale: locale, preset: .progressiveTranscription)
}

private func speechAssetStatus(for locale: Locale) async -> AssetInventory.Status {
    await AssetInventory.status(forModules: [transcriber(for: locale)])
}

private func appendSegment(_ segment: String, to text: inout String) {
    let cleaned = segment.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !cleaned.isEmpty else { return }
    if !text.isEmpty, !text.hasSuffix(" ") { text.append(" ") }
    text.append(cleaned)
}

private final class CaptureSession: @unchecked Sendable {
    private let id: String
    private let writer: OutputWriter
    private let engine: AVAudioEngine
    private let analyzer: SpeechAnalyzer
    private let inputContinuation: AsyncStream<AnalyzerInput>.Continuation
    private let analysisTask: Task<Void, Error>
    private let resultTask: Task<String, Error>
    private var stopped = false
    private let stateLock = NSLock()

    private init(
        id: String,
        writer: OutputWriter,
        engine: AVAudioEngine,
        analyzer: SpeechAnalyzer,
        inputContinuation: AsyncStream<AnalyzerInput>.Continuation,
        analysisTask: Task<Void, Error>,
        resultTask: Task<String, Error>
    ) {
        self.id = id
        self.writer = writer
        self.engine = engine
        self.analyzer = analyzer
        self.inputContinuation = inputContinuation
        self.analysisTask = analysisTask
        self.resultTask = resultTask
    }

    static func start(id: String, locale: Locale, writer: OutputWriter) async throws -> CaptureSession {
        guard await AVCaptureDevice.requestAccess(for: .audio) else {
            throw RuntimeError.microphoneDenied
        }

        let module = transcriber(for: locale)
        guard await AssetInventory.status(forModules: [module]) == .installed else {
            throw RuntimeError.speechAssetUnavailable(locale.identifier)
        }

        let engine = AVAudioEngine()
        let inputNode = engine.inputNode
        let naturalFormat = inputNode.outputFormat(forBus: 0)
        guard naturalFormat.sampleRate > 0,
              let analyzerFormat = await SpeechAnalyzer.bestAvailableAudioFormat(
                  compatibleWith: [module],
                  considering: naturalFormat
              ),
              let converter = AVAudioConverter(from: naturalFormat, to: analyzerFormat)
        else {
            throw RuntimeError.speechAssetUnavailable(locale.identifier)
        }

        let (inputStream, continuation) = AsyncStream<AnalyzerInput>.makeStream()
        let analyzer = SpeechAnalyzer(
            modules: [module],
            options: .init(priority: .userInitiated, modelRetention: .lingering)
        )
        try await analyzer.prepareToAnalyze(in: analyzerFormat)

        let resultTask = Task<String, Error> {
            var finalized = ""
            var latestVolatile = ""
            for try await result in module.results {
                try Task.checkCancellation()
                let segment = String(result.text.characters)
                if result.isFinal {
                    appendSegment(segment, to: &finalized)
                    latestVolatile = ""
                } else {
                    latestVolatile = segment.trimmingCharacters(in: .whitespacesAndNewlines)
                }
                var display = finalized
                appendSegment(latestVolatile, to: &display)
                await writer.send(
                    id: id,
                    event: "transcript",
                    payload: [
                        "text": .string(display),
                        "isFinal": .bool(false),
                        "segmentFinal": .bool(result.isFinal),
                    ]
                )
            }
            return finalized
        }

        let analysisTask = Task<Void, Error> {
            try await analyzer.start(inputSequence: inputStream)
        }

        inputNode.installTap(onBus: 0, bufferSize: 2_048, format: naturalFormat) { buffer, _ in
            let ratio = analyzerFormat.sampleRate / naturalFormat.sampleRate
            let capacity = AVAudioFrameCount(ceil(Double(buffer.frameLength) * ratio)) + 1
            guard let converted = AVAudioPCMBuffer(pcmFormat: analyzerFormat, frameCapacity: capacity) else {
                return
            }
            var conversionError: NSError?
            let input = ConverterInput()
            let status = converter.convert(to: converted, error: &conversionError) { _, outputStatus in
                guard let buffer = input.take(buffer) else {
                    outputStatus.pointee = .noDataNow
                    return nil
                }
                outputStatus.pointee = .haveData
                return buffer
            }
            if status != .error, conversionError == nil, converted.frameLength > 0 {
                continuation.yield(AnalyzerInput(buffer: converted))
            }
        }
        engine.prepare()
        try engine.start()

        return CaptureSession(
            id: id,
            writer: writer,
            engine: engine,
            analyzer: analyzer,
            inputContinuation: continuation,
            analysisTask: analysisTask,
            resultTask: resultTask
        )
    }

    func finish() async throws -> String {
        let shouldStop = stateLock.withLock {
            let shouldStop = !stopped
            stopped = true
            return shouldStop
        }
        guard shouldStop else { throw RuntimeError.captureNotActive }

        engine.stop()
        engine.inputNode.removeTap(onBus: 0)
        inputContinuation.finish()
        try await analyzer.finalizeAndFinishThroughEndOfInput()
        try await analysisTask.value
        let transcript = try await resultTask.value
        await writer.send(
            id: id,
            event: "transcript",
            payload: ["text": .string(transcript), "isFinal": .bool(true)]
        )
        return transcript
    }

    func cancel() async {
        let shouldStop = stateLock.withLock {
            let shouldStop = !stopped
            stopped = true
            return shouldStop
        }
        guard shouldStop else { return }

        engine.stop()
        engine.inputNode.removeTap(onBus: 0)
        inputContinuation.finish()
        analysisTask.cancel()
        resultTask.cancel()
        await analyzer.cancelAndFinishNow()
    }
}

@Generable
private enum RouteKind {
    case local
    case cloud
}

@Generable
private struct RouteAssessment {
    @Guide(description: "Whether the request is safe and suitable for the small on-device model.")
    let route: RouteKind

    @Guide(description: "One short snake_case reason code.")
    let reasonCode: String
}

@MainActor
private final class NativeSpeechOutput: NSObject, AVSpeechSynthesizerDelegate {
    private let synthesizer = AVSpeechSynthesizer()

    override init() {
        super.init()
        synthesizer.delegate = self
    }

    func speak(_ text: String, locale: String?) {
        guard !text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return }
        let utterance = AVSpeechUtterance(string: text)
        if let locale, let voice = AVSpeechSynthesisVoice(language: locale) {
            utterance.voice = voice
        }
        synthesizer.speak(utterance)
    }

    func cancel() {
        synthesizer.stopSpeaking(at: .immediate)
    }
}

private actor AppleRuntime {
    private let writer: OutputWriter
    private let speechOutput: NativeSpeechOutput
    private var capture: CaptureSession?
    private var generationTasks: [String: Task<Void, Never>] = [:]
    private var sessions: [String: (session: LanguageModelSession, usedAt: ContinuousClock.Instant)] = [:]
    private var idleReleaseTask: Task<Void, Never>?
    private let clock = ContinuousClock()

    init(writer: OutputWriter, speechOutput: NativeSpeechOutput) {
        self.writer = writer
        self.speechOutput = speechOutput
    }

    func handle(_ command: CommandEnvelope) {
        switch command.command {
        case "status":
            Task { await sendStatus(id: command.id, localeIdentifier: command.locale) }
        case "install_speech_assets":
            Task { await installSpeechAssets(id: command.id, localeIdentifier: command.locale) }
        case "start_capture":
            Task { await startCapture(id: command.id, localeIdentifier: command.locale) }
        case "stop_capture":
            Task { await stopCapture(id: command.id) }
        case "cancel_capture":
            Task { await cancelCapture(id: command.id) }
        case "assess":
            Task { await assess(command) }
        case "generate":
            startGeneration(command)
        case "cancel_generation":
            generationTasks[command.conversationId ?? command.id]?.cancel()
            Task { await writer.send(id: command.id, event: "result") }
        case "speak":
            let text = command.text ?? ""
            let locale = command.locale
            Task { @MainActor in speechOutput.speak(text, locale: locale) }
            Task { await writer.send(id: command.id, event: "result") }
        case "cancel_speech":
            Task { @MainActor in speechOutput.cancel() }
            Task { await writer.send(id: command.id, event: "result") }
        case "release_idle_resources":
            releaseIdleSessions(force: true)
            Task {
                await SpeechModels.endRetention()
                await writer.send(id: command.id, event: "result")
            }
        case "shutdown":
            Task {
                idleReleaseTask?.cancel()
                await capture?.cancel()
                capture = nil
                generationTasks.values.forEach { $0.cancel() }
                await MainActor.run { speechOutput.cancel() }
                await SpeechModels.endRetention()
                await writer.send(id: command.id, event: "result")
                Foundation.exit(0)
            }
        default:
            Task { await sendError(id: command.id, error: RuntimeError.invalidCommand(command.command)) }
        }
    }

    private func sendStatus(id: String, localeIdentifier: String?) async {
        do {
            let locale = try await preferredLocale(localeIdentifier)
            let assetStatus = await speechAssetStatus(for: locale)
            let model = appleModelAvailability()
            await writer.send(
                id: id,
                event: "result",
                payload: [
                    "platform": .string("macos"),
                    "locale": .string(locale.identifier),
                    "speechAvailable": .bool(SpeechTranscriber.isAvailable),
                    "speechAssetStatus": .string(String(describing: assetStatus)),
                    "modelAvailable": .bool(model.available),
                    "modelReason": .string(model.reason),
                ]
            )
        } catch {
            await sendError(id: id, error: error)
        }
    }

    private func installSpeechAssets(id: String, localeIdentifier: String?) async {
        do {
            let locale = try await preferredLocale(localeIdentifier)
            let module = transcriber(for: locale)
            let initial = await AssetInventory.status(forModules: [module])
            if initial == .installed {
                await writer.send(id: id, event: "result", payload: ["locale": .string(locale.identifier)])
                return
            }
            guard initial != .unsupported else {
                throw RuntimeError.speechAssetUnavailable(locale.identifier)
            }
            _ = try await AssetInventory.reserve(locale: locale)
            guard let request = try await AssetInventory.assetInstallationRequest(supporting: [module]) else {
                await writer.send(id: id, event: "result", payload: ["locale": .string(locale.identifier)])
                return
            }
            let progressTask = Task {
                while !Task.isCancelled, !request.progress.isFinished {
                    await writer.send(
                        id: id,
                        event: "asset_progress",
                        payload: ["fraction": .number(request.progress.fractionCompleted)]
                    )
                    try? await Task.sleep(for: .milliseconds(150))
                }
            }
            defer { progressTask.cancel() }
            try await request.downloadAndInstall()
            await writer.send(
                id: id,
                event: "result",
                payload: ["locale": .string(locale.identifier), "fraction": .number(1)]
            )
        } catch {
            await sendError(id: id, error: error)
        }
    }

    private func startCapture(id: String, localeIdentifier: String?) async {
        do {
            idleReleaseTask?.cancel()
            guard capture == nil else { throw RuntimeError.captureAlreadyActive }
            await MainActor.run { speechOutput.cancel() }
            let locale = try await preferredLocale(localeIdentifier)
            capture = try await CaptureSession.start(id: id, locale: locale, writer: writer)
            await writer.send(id: id, event: "result", payload: ["locale": .string(locale.identifier)])
        } catch {
            await sendError(id: id, error: error)
        }
    }

    private func stopCapture(id: String) async {
        guard let active = capture else {
            await sendError(id: id, error: RuntimeError.captureNotActive)
            return
        }
        capture = nil
        do {
            let transcript = try await active.finish()
            await writer.send(id: id, event: "result", payload: ["text": .string(transcript)])
        } catch {
            await sendError(id: id, error: error)
        }
    }

    private func cancelCapture(id: String) async {
        await capture?.cancel()
        capture = nil
        await writer.send(id: id, event: "result")
    }

    private func modelSession(for conversationId: String, instructions: String?) throws -> LanguageModelSession {
        idleReleaseTask?.cancel()
        releaseIdleSessions(force: false)
        if let entry = sessions[conversationId] {
            sessions[conversationId] = (entry.session, clock.now)
            return entry.session
        }
        let availability = appleModelAvailability()
        guard availability.available else { throw RuntimeError.modelUnavailable(availability.reason) }
        let session = LanguageModelSession(
            model: .default,
            tools: [],
            instructions: instructions ?? "Be concise, operational, and honest about uncertainty."
        )
        session.prewarm()
        sessions[conversationId] = (session, clock.now)
        return session
    }

    private func releaseIdleSessions(force: Bool) {
        let cutoff = clock.now - .seconds(300)
        sessions = sessions.filter { !force && $0.value.usedAt >= cutoff }
    }

    private func scheduleIdleRelease() {
        idleReleaseTask?.cancel()
        idleReleaseTask = Task { [weak self] in
            try? await Task.sleep(for: .seconds(300))
            guard !Task.isCancelled else { return }
            await self?.releaseIdleResources()
        }
    }

    private func releaseIdleResources() async {
        releaseIdleSessions(force: true)
        await SpeechModels.endRetention()
        idleReleaseTask = nil
    }

    private func assess(_ command: CommandEnvelope) async {
        do {
            let session = try modelSession(for: "route-assessment", instructions: "Classify routing conservatively.")
            let request = command.prompt ?? ""
            let prompt = """
            Decide whether this request suits the on-device model. Choose cloud for current events, web research, repository-wide coding, long context, advanced factual reasoning, or facts not supplied by the user. Choose local for intent classification, extraction, rewriting, summarization of supplied text, short conversation, and simple planning.

            Request: \(request)
            """
            let response = try await session.respond(to: prompt, generating: RouteAssessment.self)
            let route: String
            switch response.content.route {
            case .local: route = "local"
            case .cloud: route = "cloud"
            }
            await writer.send(
                id: command.id,
                event: "result",
                payload: [
                    "route": .string(route),
                    "reasonCode": .string(response.content.reasonCode),
                ]
            )
            scheduleIdleRelease()
        } catch {
            await sendError(id: command.id, error: error)
        }
    }

    private func startGeneration(_ command: CommandEnvelope) {
        let conversationId = command.conversationId ?? command.id
        generationTasks[conversationId]?.cancel()
        generationTasks[conversationId] = Task { [writer] in
            do {
                let session = try modelSession(for: conversationId, instructions: command.instructions)
                var previous = ""
                let stream = session.streamResponse(to: command.prompt ?? "")
                for try await snapshot in stream {
                    try Task.checkCancellation()
                    let current = snapshot.content
                    guard current.hasPrefix(previous) else { continue }
                    let delta = String(current.dropFirst(previous.count))
                    previous = current
                    if !delta.isEmpty {
                        await writer.send(
                            id: command.id,
                            event: "model_snapshot",
                            payload: ["text": .string(current), "delta": .string(delta)]
                        )
                    }
                }
                await writer.send(
                    id: command.id,
                    event: "result",
                    payload: ["text": .string(previous)]
                )
                scheduleIdleRelease()
            } catch is CancellationError {
                await writer.send(
                    id: command.id,
                    event: "error",
                    code: "cancelled",
                    message: "Generation cancelled."
                )
            } catch {
                await sendError(id: command.id, error: error)
            }
            generationTasks[conversationId] = nil
        }
    }

    private func sendError(id: String, error: Error) async {
        let code: String
        if let runtimeError = error as? RuntimeError {
            switch runtimeError {
            case .microphoneDenied: code = "microphone_denied"
            case .modelUnavailable: code = "model_unavailable"
            case .speechAssetUnavailable: code = "speech_asset_unavailable"
            case .unsupportedLocale: code = "unsupported_locale"
            case .captureAlreadyActive: code = "capture_active"
            case .captureNotActive: code = "capture_inactive"
            case .invalidCommand: code = "invalid_command"
            }
        } else if error is CancellationError {
            code = "cancelled"
        } else {
            code = "apple_runtime_error"
        }
        await writer.send(
            id: id,
            event: "error",
            code: code,
            message: error.localizedDescription
        )
    }
}

@main
private struct FlowStateAppleRuntimeMain {
    static func main() async {
        let writer = OutputWriter()
        let speechOutput = await MainActor.run { NativeSpeechOutput() }
        let runtime = AppleRuntime(writer: writer, speechOutput: speechOutput)
        let decoder = JSONDecoder()

        while let line = readLine() {
            guard let data = line.data(using: .utf8) else { continue }
            do {
                let command = try decoder.decode(CommandEnvelope.self, from: data)
                await runtime.handle(command)
            } catch {
                await writer.send(
                    id: "invalid",
                    event: "error",
                    code: "invalid_json",
                    message: error.localizedDescription
                )
            }
        }
    }
}
