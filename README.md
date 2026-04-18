# Net Skip

Net Skip is a web browser app for iOS and Android.


<p style="vertical-align: middle; margin-left: auto; margin-right: auto; text-align: center;">
<a href="https://apps.apple.com/us/app/net-skip/id1640618584?itscg=30200&amp;itsct=apps_box_appicon" style="width: 170px; height: 170px; border-radius: 22%; overflow: hidden; display: inline-block; vertical-align: middle; margin-left: auto; margin-right: auto; text-align: center;">
  <img src="https://appfair.org/img/net-skip-icon.jpg" alt="Net Skip" style="width: 170px; height: 170px; border-radius: 22%; overflow: hidden; display: inline-block; vertical-align: middle;" />
</a>
</p>

<p style="vertical-align: middle; margin-left: auto; margin-right: auto; text-align: center;">
<a href="https://apps.apple.com/us/app/net-skip/id1640618584?itscg=30200&amp;itsct=apps_box_appicon" stylex="width: 170px; height: 170px; border-radius: 22%; overflow: hidden; display: inline-block; vertical-align: middle; margin-left: auto; margin-right: auto; text-align: center;">
  <img alt="Download on App Store Button" src="https://appfair.org/img/download-app-store.png" style="width: 170px; display: inline-block; vertical-align: middle;" />
</a>
</p>


## Building

This project is both a stand-alone Swift Package Manager module,
as well as an Xcode project that builds and translates the project
into a Kotlin Gradle project for Android using the skipstone plugin.

## Testing

The module can be tested using the standard `swift test` command
or by running the test target for the macOS desintation in Xcode,
which will run the Swift tests as well as the transpiled
Kotlin JUnit tests in the Robolectric Android simulation environment.

Parity testing can be performed with `skip test`,
which will output a table of the test results for both platforms.

## Running

Xcode and Android Studio must be downloaded and installed in order to
run the app in the iOS simulator / Android emulator.
An Android emulator must already be running, which can be launched from 
Android Studio's Device Manager.

To run both the Swift and Kotlin apps simultaneously, 
launch the NetSkipApp target from Xcode.
A build phases runs the "Launch Android APK" script that
will deploy the transpiled app a running Android emulator or connected device.
Logging output for the iOS app can be viewed in the Xcode console, and in
Android Studio's logcat tab for the transpiled Kotlin app.
