import Foundation

// Dummy protocol for compilation purposes if it doesn't already exist
public protocol ScannerPlugin {
    var name: String { get }
    func scan(filePath: String) async throws -> ScanResult
}

public struct ScanResult {
    public let filePath: String
    public let type: String
    public let isExecutable: Bool
    public let isSensitive: Bool
    
    public init(filePath: String, type: String, isExecutable: Bool, isSensitive: Bool) {
        self.filePath = filePath
        self.type = type
        self.isExecutable = isExecutable
        self.isSensitive = isSensitive
    }
}

public enum ScannerError: Error {
    case executionFailed(String)
    case fileNotFound
}

public class MagikaScanner: ScannerPlugin {
    public let name = "Magika"
    
    public init() {}
    
    public func scan(filePath: String) async throws -> ScanResult {
        let fileManager = FileManager.default
        guard fileManager.fileExists(atPath: filePath) else {
            throw ScannerError.fileNotFound
        }
        
        let process = Process()
        let pipe = Pipe()
        let errorPipe = Pipe()
        
        // We assume magika is in the PATH. Use env to locate it.
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
        
        // Magika default output format is usually: "path/to/file: file type description"
        let typeDescription: String
        if let colonIndex = output.firstIndex(of: ":") {
            let afterColon = output.index(after: colonIndex)
            typeDescription = String(output[afterColon...]).trimmingCharacters(in: .whitespacesAndNewlines)
        } else {
            typeDescription = output
        }
        
        let lowerType = typeDescription.lowercased()
        
        // Basic heuristic for executables
        let isExecutable = lowerType.contains("executable") || 
                           lowerType.contains("mach-o") || 
                           lowerType.contains("elf") || 
                           lowerType.contains("pe32") ||
                           lowerType.contains("script")
                           
        // Basic heuristic for sensitive files
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
