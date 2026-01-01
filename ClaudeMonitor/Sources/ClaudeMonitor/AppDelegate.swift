import Cocoa
import Foundation
import UserNotifications

class AppDelegate: NSObject, NSApplicationDelegate {
    private var statusItem: NSStatusItem!
    private var logWatcher: LogWatcher?
    private var sessions: [String: SessionInfo] = [:]
    private var recentEvents: [EventInfo] = []

    private let logDir = FileManager.default.homeDirectoryForCurrentUser
        .appendingPathComponent(".claude-monitor/logs")

    func applicationDidFinishLaunching(_ notification: Notification) {
        setupStatusItem()
        setupLogWatcher()
        loadExistingEvents()
        updateMenu()
    }

    private func setupStatusItem() {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)

        if let button = statusItem.button {
            button.image = NSImage(systemSymbolName: "terminal", accessibilityDescription: "Claude Monitor")
            button.image?.isTemplate = true
        }
    }

    private func setupLogWatcher() {
        let eventsFile = logDir.appendingPathComponent("events.jsonl")
        logWatcher = LogWatcher(filePath: eventsFile) { [weak self] newLine in
            self?.handleNewEvent(newLine)
        }
        logWatcher?.start()
    }

    private func loadExistingEvents() {
        let eventsFile = logDir.appendingPathComponent("events.jsonl")
        guard let content = try? String(contentsOf: eventsFile, encoding: .utf8) else { return }

        let lines = content.components(separatedBy: .newlines).filter { !$0.isEmpty }
        for line in lines.suffix(50) {
            if let event = parseEvent(line) {
                processEvent(event)
            }
        }
    }

    private func handleNewEvent(_ line: String) {
        DispatchQueue.main.async { [weak self] in
            if let event = self?.parseEvent(line) {
                self?.processEvent(event)
                self?.updateMenu()
                self?.showNotificationIfNeeded(event)
            }
        }
    }

    private func parseEvent(_ line: String) -> EventInfo? {
        guard let data = line.data(using: .utf8),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            return nil
        }

        return EventInfo(
            timestamp: json["timestamp"] as? String ?? "",
            event: json["event"] as? String ?? "",
            matcher: json["matcher"] as? String ?? "",
            projectName: json["project_name"] as? String ?? "unknown",
            projectDir: json["project_dir"] as? String ?? "",
            sessionId: json["session_id"] as? String ?? "",
            message: json["message"] as? String ?? "",
            notificationType: json["notification_type"] as? String ?? ""
        )
    }

    private func processEvent(_ event: EventInfo) {
        recentEvents.append(event)
        if recentEvents.count > 20 {
            recentEvents.removeFirst()
        }

        let key = event.projectDir.isEmpty ? event.projectName : event.projectDir

        switch event.event {
        case "session_start":
            sessions[key] = SessionInfo(
                projectName: event.projectName,
                projectDir: event.projectDir,
                sessionId: event.sessionId,
                status: .active,
                lastEvent: event.timestamp
            )
        case "session_end":
            sessions.removeValue(forKey: key)
        case "notification":
            if var session = sessions[key] {
                if event.notificationType == "permission_prompt" {
                    session.status = .waitingPermission
                } else if event.notificationType == "idle_prompt" {
                    session.status = .waitingInput
                }
                session.lastEvent = event.timestamp
                sessions[key] = session
            }
        case "stop":
            if var session = sessions[key] {
                session.status = .completed
                session.lastEvent = event.timestamp
                sessions[key] = session
            }
        default:
            if var session = sessions[key] {
                session.lastEvent = event.timestamp
                sessions[key] = session
            }
        }
    }

    private func showNotificationIfNeeded(_ event: EventInfo) {
        guard event.event == "notification" else { return }

        let content = UNMutableNotificationContent()
        content.title = "Claude Code - \(event.projectName)"

        switch event.notificationType {
        case "permission_prompt":
            content.body = "Permission required"
            content.sound = .default
        case "idle_prompt":
            content.body = "Waiting for input"
            content.sound = .default
        default:
            return
        }

        let request = UNNotificationRequest(
            identifier: UUID().uuidString,
            content: content,
            trigger: nil
        )

        UNUserNotificationCenter.current().add(request)
    }

    private func updateMenu() {
        let menu = NSMenu()

        // Status header
        let waitingCount = sessions.values.filter {
            $0.status == .waitingPermission || $0.status == .waitingInput
        }.count

        if waitingCount > 0 {
            statusItem.button?.image = NSImage(systemSymbolName: "terminal.fill", accessibilityDescription: "Claude Monitor - Action Required")
            let headerItem = NSMenuItem(title: "\(waitingCount) session(s) waiting", action: nil, keyEquivalent: "")
            headerItem.attributedTitle = NSAttributedString(
                string: "\(waitingCount) session(s) waiting",
                attributes: [.foregroundColor: NSColor.systemOrange, .font: NSFont.boldSystemFont(ofSize: 13)]
            )
            menu.addItem(headerItem)
        } else if sessions.isEmpty {
            statusItem.button?.image = NSImage(systemSymbolName: "terminal", accessibilityDescription: "Claude Monitor")
            menu.addItem(NSMenuItem(title: "No active sessions", action: nil, keyEquivalent: ""))
        } else {
            statusItem.button?.image = NSImage(systemSymbolName: "terminal", accessibilityDescription: "Claude Monitor")
            menu.addItem(NSMenuItem(title: "\(sessions.count) active session(s)", action: nil, keyEquivalent: ""))
        }

        menu.addItem(NSMenuItem.separator())

        // Active sessions
        if !sessions.isEmpty {
            menu.addItem(NSMenuItem(title: "Sessions", action: nil, keyEquivalent: ""))

            for (_, session) in sessions.sorted(by: { $0.value.lastEvent > $1.value.lastEvent }) {
                let emoji = session.status.emoji
                let title = "\(emoji) \(session.projectName)"
                let item = NSMenuItem(title: title, action: nil, keyEquivalent: "")

                let submenu = NSMenu()
                submenu.addItem(NSMenuItem(title: "Status: \(session.status.description)", action: nil, keyEquivalent: ""))
                submenu.addItem(NSMenuItem(title: "Last: \(formatTime(session.lastEvent))", action: nil, keyEquivalent: ""))
                if !session.projectDir.isEmpty {
                    submenu.addItem(NSMenuItem(title: session.projectDir, action: nil, keyEquivalent: ""))
                }
                item.submenu = submenu

                menu.addItem(item)
            }

            menu.addItem(NSMenuItem.separator())
        }

        // Recent events
        if !recentEvents.isEmpty {
            let eventsItem = NSMenuItem(title: "Recent Events", action: nil, keyEquivalent: "")
            let eventsSubmenu = NSMenu()

            for event in recentEvents.suffix(10).reversed() {
                let emoji = eventEmoji(event)
                let title = "\(emoji) \(event.projectName): \(event.event)"
                let item = NSMenuItem(title: title, action: nil, keyEquivalent: "")
                eventsSubmenu.addItem(item)
            }

            eventsItem.submenu = eventsSubmenu
            menu.addItem(eventsItem)
            menu.addItem(NSMenuItem.separator())
        }

        // Actions
        menu.addItem(NSMenuItem(title: "Open Log Folder", action: #selector(openLogFolder), keyEquivalent: "l"))
        menu.addItem(NSMenuItem(title: "Clear Sessions", action: #selector(clearSessions), keyEquivalent: ""))
        menu.addItem(NSMenuItem.separator())
        menu.addItem(NSMenuItem(title: "Quit", action: #selector(quit), keyEquivalent: "q"))

        statusItem.menu = menu
    }

    private func formatTime(_ timestamp: String) -> String {
        // Simple extraction of time from ISO timestamp
        if let range = timestamp.range(of: "T") {
            let time = String(timestamp[range.upperBound...]).prefix(8)
            return String(time)
        }
        return timestamp
    }

    private func eventEmoji(_ event: EventInfo) -> String {
        switch event.event {
        case "notification":
            switch event.notificationType {
            case "permission_prompt": return "🔐"
            case "idle_prompt": return "⏳"
            default: return "🔔"
            }
        case "stop": return "✅"
        case "session_start": return "🚀"
        case "session_end": return "🏁"
        default: return "📌"
        }
    }

    @objc private func openLogFolder() {
        NSWorkspace.shared.open(logDir)
    }

    @objc private func clearSessions() {
        sessions.removeAll()
        updateMenu()
    }

    @objc private func quit() {
        NSApplication.shared.terminate(nil)
    }
}

// MARK: - Models

struct SessionInfo {
    let projectName: String
    let projectDir: String
    let sessionId: String
    var status: SessionStatus
    var lastEvent: String
}

enum SessionStatus {
    case active
    case waitingPermission
    case waitingInput
    case completed

    var emoji: String {
        switch self {
        case .active: return "🟢"
        case .waitingPermission: return "🔐"
        case .waitingInput: return "⏳"
        case .completed: return "✅"
        }
    }

    var description: String {
        switch self {
        case .active: return "Active"
        case .waitingPermission: return "Waiting for permission"
        case .waitingInput: return "Waiting for input"
        case .completed: return "Completed"
        }
    }
}

struct EventInfo {
    let timestamp: String
    let event: String
    let matcher: String
    let projectName: String
    let projectDir: String
    let sessionId: String
    let message: String
    let notificationType: String
}
