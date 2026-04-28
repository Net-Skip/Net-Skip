// SPDX-License-Identifier: GPL-2.0-or-later
import Testing
import Foundation
@testable import NetSkipMiniApp

@Suite struct NetSkipMiniAppTests {
    @Test func sampleCatalog() {
        #expect(sampleMiniApps.count == 5)
        #expect(sampleMiniApps[0].id == "showcase-demo")
    }
}
