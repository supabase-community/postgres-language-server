import XCTest
import SwiftTreeSitter
import TreeSitterPgls

final class TreeSitterPglsTests: XCTestCase {
    func testCanLoadGrammar() throws {
        let parser = Parser()
        let language = Language(language: tree_sitter_pgls())
        XCTAssertNoThrow(try parser.setLanguage(language),
                         "Error loading Postgres Language Server grammar")
    }
}
