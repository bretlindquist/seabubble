import Foundation
import IslandShared

public enum ScannerError: Error {
    case executionFailed(String)
    case fileNotFound
}

public final class MagikaScanner: ScannerPlugin {
    public let id = "com.island.scanner.magika"
    public let name = "Magika File Scanner"
    
    public init() {}
    
    public func evaluate(request: CapabilityRequest, context: StatefulScanContext) async throws -> ScannerResult {
        // Only run Magika if the capability involves a file read, execute, or write
        // For now, let's just extract the file path from the payload.
        // A naive heuristic: if payload contains a path, scan it.
        // In a real scenario we'd parse the capability properly.
        let components = request.payload.components(separatedBy: .whitespaces)
        var evidence: [String] = []
        var maxRisk = 0
        var interventionRequired = false
        
        for component in components {
            let filePath = component.trimmingCharacters(in: CharacterSet(charactersIn: "\"'"))
            guard filePath.hasPrefix("/") || filePath.hasPrefix("./") else { continue }
            
            // Resolve absolute path if relative to cwd
            let resolvedPath = filePath.hasPrefix("./") ? 
                request.cwd + String(filePath.dropFirst(1)) : filePath
                
            let fileManager = FileManager.default
            if fileManager.fileExists(atPath: resolvedPath) {
                do {
                    let result = try await scan(filePath: resolvedPath)
                    evidence.append("Magika found '\(result.type)' at \(resolvedPath)")
                    if result.isExecutable {
                        evidence.append("Executable file detected.")
                        maxRisk = max(maxRisk, 30)
                    }
                    if result.isSensitive {
                        evidence.append("Sensitive file detected.")
                        maxRisk = max(maxRisk, 80)
                        interventionRequired = true
                    }
                } catch {
                    evidence.append("Magika scan failed for \(resolvedPath): \(error.localizedDescription)")
                }
            }
        }
        
        return ScannerResult(
            riskModifier: maxRisk,
            requiresIntervention: interventionRequired,
            evidence: evidence,
            suggestedSeverity: interventionRequired ? .high : nil
        )
    }
    
    public struct ScanResult {
        public let filePath: String
        public let type: String
        public let isExecutable: Bool
        public let isSensitive: Bool
    }
    
    private func scan(filePath: String) async throws -> ScanResult {
        let process = Process()
        let pipe = Pipe()
        let errorPipe = Pipe()
        
        process.executableURL = URL(fileURLWithPath: "/usr/bin/env")
        process.arguments = ["magika", filePath]
        process.standardOutput = pipe
        process.standardError = errorPipe
        
        do {
            try process.run()
            process.waitUntilExit()
        } catch {
            throw ScannerError.executionFailed("Failed to execute magika: \(error.localizedDescription)")
        }
        
        if process.terminationStatus != 0 {
            let errorData = errorPipe.fileHandleForReading.readDataToEndOfFile()
            let errorString = String(data: errorData, encoding: .utf8)?.trimmingCharacters(in: .whitespacesAndNewlines) ?? "Unknown error"
            throw ScannerError.executionFailed("magika exited with status \(process.terminationStatus): \(errorString)")
        }
        
        let data = pipe.fileHandleForReading.readDataToEndOfFile()
        guard let output = String(data: data, encoding: .utf8)?.trimmingCharacters(in: .whitespacesAndNewlines) else {
            throw ScannerError.executionFailed("Could not read magika output")
        }
        
        let typeDescription: String
        if let colonIndex = output.firstIndex(of: ":") {
            let afterColon = output.index(after: colonIndex)
            typeDescription = String(output[afterColon...]).trimmingCharacters(in: .whitespacesAndNewlines)
        } else {
            typeDescription = output
        }
        
        let lowerType = typeDescription.lowercased()
        
        let isExecutable = lowerType.contains("executable") || 
                           lowerType.contains("mach-o") || 
                           lowerType.contains("elf") || 
                           lowerType.contains("pe32") ||
                           lowerType.contains("script")
                           
        let isSensitive = lowerType.contains("private key") || 
                          lowerType.contains("pem") || 
                          lowerType.contains("certificate") || 
                          lowerType.contains("keystore")
        
        return ScanResult(
            filePath: filePath,
            type: typeDescription,
            isExecutable: isExecutable,
            isSensitive: isSensitive
        )
    }
}
