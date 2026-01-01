import Foundation

class LogWatcher {
    private let filePath: URL
    private let onNewLine: (String) -> Void
    private var timer: Timer?
    private var lastOffset: UInt64 = 0
    private var lastModified: Date?

    init(filePath: URL, onNewLine: @escaping (String) -> Void) {
        self.filePath = filePath
        self.onNewLine = onNewLine
    }

    func start() {
        // Create directory and file if needed
        let dir = filePath.deletingLastPathComponent()
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)

        if !FileManager.default.fileExists(atPath: filePath.path) {
            FileManager.default.createFile(atPath: filePath.path, contents: nil)
        }

        // Get initial file size
        updateLastOffset()

        // Start polling timer (every 0.5 seconds)
        timer = Timer.scheduledTimer(withTimeInterval: 0.5, repeats: true) { [weak self] _ in
            self?.checkForChanges()
        }
        RunLoop.main.add(timer!, forMode: .common)
    }

    func stop() {
        timer?.invalidate()
        timer = nil
    }

    private func updateLastOffset() {
        if let attrs = try? FileManager.default.attributesOfItem(atPath: filePath.path) {
            lastOffset = attrs[.size] as? UInt64 ?? 0
            lastModified = attrs[.modificationDate] as? Date
        }
    }

    private func checkForChanges() {
        guard let attrs = try? FileManager.default.attributesOfItem(atPath: filePath.path),
              let currentSize = attrs[.size] as? UInt64,
              let modDate = attrs[.modificationDate] as? Date else {
            return
        }

        // Check if file was modified
        if let lastMod = lastModified, modDate <= lastMod, currentSize == lastOffset {
            return
        }

        // File was truncated or recreated
        if currentSize < lastOffset {
            lastOffset = 0
        }

        // Read new content
        if currentSize > lastOffset {
            readNewContent(from: lastOffset, to: currentSize)
            lastOffset = currentSize
            lastModified = modDate
        }
    }

    private func readNewContent(from startOffset: UInt64, to endOffset: UInt64) {
        guard let handle = FileHandle(forReadingAtPath: filePath.path) else { return }
        defer { try? handle.close() }

        do {
            try handle.seek(toOffset: startOffset)
            let length = Int(endOffset - startOffset)
            guard let data = try handle.read(upToCount: length),
                  let content = String(data: data, encoding: .utf8) else {
                return
            }

            let lines = content.components(separatedBy: .newlines)
            for line in lines where !line.isEmpty {
                onNewLine(line)
            }
        } catch {
            // Ignore errors
        }
    }
}
