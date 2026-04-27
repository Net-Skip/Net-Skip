#!/bin/sh -ex
# buld with local resources
export NET_SKIP_DEPENDENCY_ROOT=/opt/src/github/skiptools

cd $(dirname $(realpath $0))/..

PRODUCT_NAME=$(grep '^PRODUCT_NAME = ' Skip.env | tr -d ' ' | cut -f 2- -d '=')
PRODUCT_BUNDLE_IDENTIFIER=$(grep '^PRODUCT_BUNDLE_IDENTIFIER = ' Skip.env | tr -d ' ' | cut -f 2- -d '=')

xcodebuild -workspace Project.xcworkspace -scheme "${PRODUCT_NAME}" -sdk iphonesimulator -skipPackagePluginValidation -skipMacroValidation -derivedDataPath .build/Darwin/DerivedData
xcrun simctl install booted $(find .build/Darwin/DerivedData -name "*.app")
xcrun simctl launch booted "${PRODUCT_BUNDLE_IDENTIFIER}"
