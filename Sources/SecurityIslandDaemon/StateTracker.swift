import Foundation

public actor StateTracker {
    public var downloadedFiles: Set<String> = []
    public var executedScripts: Set<String> = []
    
    public init() {}
    
    public func recordDownload(path: String) {
        downloadedFiles.insert(path)
    }
    
    public func isDownloaded(path: String) -> Bool {
        return downloadedFiles.contains(path)
    }
    
    public func recordExecution(path: String) {
        executedScripts.insert(path)
    }
}
