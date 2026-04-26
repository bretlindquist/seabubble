import Foundation

public enum Token: Equatable {
    case word(String)
    case pipe
    case and
    case or
    case semi
}

public struct Command {
    public var args: [String]
}

public struct Pipeline {
    public var commands: [Command]
}

public struct Ast {
    public var pipelines: [Pipeline]
}

public func tokenize(_ input: String) -> [Token] {
    var tokens: [Token] = []
    let chars = Array(input)
    var idx = 0
    var currentWord = ""
    var inWord = false
    var inSingleQuote = false
    var inDoubleQuote = false

    func pushWord() {
        if inWord {
            tokens.append(.word(currentWord))
            currentWord = ""
            inWord = false
        }
    }
    
    func peek() -> Character? {
        if idx + 1 < chars.count { return chars[idx + 1] }
        return nil
    }

    while idx < chars.count {
        let c = chars[idx]
        
        if inSingleQuote {
            if c == "'" {
                inSingleQuote = false
            } else {
                currentWord.append(c)
            }
            idx += 1
            continue
        }

        if inDoubleQuote {
            if c == "\"" {
                inDoubleQuote = false
            } else if c == "\\" {
                if let next = peek() {
                    currentWord.append(next)
                    idx += 1
                }
            } else {
                currentWord.append(c)
            }
            idx += 1
            continue
        }

        switch c {
        case "'":
            inWord = true
            inSingleQuote = true
        case "\"":
            inWord = true
            inDoubleQuote = true
        case "\\":
            inWord = true
            if let next = peek() {
                currentWord.append(next)
                idx += 1
            }
        case "|":
            pushWord()
            if peek() == "|" {
                idx += 1
                tokens.append(.or)
            } else {
                tokens.append(.pipe)
            }
        case "&":
            if peek() == "&" {
                idx += 1
                pushWord()
                tokens.append(.and)
            } else {
                inWord = true
                currentWord.append("&")
            }
        case ";":
            pushWord()
            tokens.append(.semi)
        case _ where c.isWhitespace:
            pushWord()
        default:
            inWord = true
            currentWord.append(c)
        }
        idx += 1
    }
    pushWord()
    return tokens
}

public func parse(_ tokens: [Token]) -> Ast {
    var pipelines: [Pipeline] = []
    var currentPipeline = Pipeline(commands: [])
    var currentCmd = Command(args: [])

    for token in tokens {
        switch token {
        case .word(let w):
            currentCmd.args.append(w)
        case .pipe:
            if !currentCmd.args.isEmpty {
                currentPipeline.commands.append(currentCmd)
                currentCmd = Command(args: [])
            }
        case .and, .or, .semi:
            if !currentCmd.args.isEmpty {
                currentPipeline.commands.append(currentCmd)
                currentCmd = Command(args: [])
            }
            if !currentPipeline.commands.isEmpty {
                pipelines.append(currentPipeline)
                currentPipeline = Pipeline(commands: [])
            }
        }
    }

    if !currentCmd.args.isEmpty {
        currentPipeline.commands.append(currentCmd)
    }
    if !currentPipeline.commands.isEmpty {
        pipelines.append(currentPipeline)
    }

    return Ast(pipelines: pipelines)
}
