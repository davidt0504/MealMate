import java.util.Properties

plugins {
    id("com.android.application")
    // The Flutter Gradle Plugin must be applied after the Android and Kotlin Gradle plugins.
    id("dev.flutter.flutter-gradle-plugin")
}

// Release signing comes from android/key.properties (gitignored) locally and from MEALMATE_*
// environment variables in CI. Never from signingConfigs.debug: AGP silently generates a
// throwaway debug key wherever it fails to find one, which is how v1.0.0 through v1.0.3 each
// shipped under a different certificate without failing a build.
val keystoreProperties = Properties().apply {
    val file = rootProject.file("key.properties")
    if (file.exists()) {
        file.inputStream().use { load(it) }
    }
}

fun signingSetting(property: String, variable: String): String? =
    System.getenv(variable) ?: keystoreProperties.getProperty(property)

// Absent on a fresh clone with no key configured; present in CI and on the owner's machine.
val releaseKeystore: String? = signingSetting("storeFile", "MEALMATE_KEYSTORE")

android {
    namespace = "dev.mealmate.temp"
    compileSdk = flutter.compileSdkVersion
    ndkVersion = flutter.ndkVersion

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    defaultConfig {
        // Temporary development identity (DEC-001 §1). The production application ID is
        // DEC-004's decision and must replace this before MVP-018, MVP-021, or MVP-022
        // binds an identifier to Firebase, App Links, signing, or Play.
        applicationId = "dev.mealmate.temp"
        // You can update the following values to match your application needs.
        // For more information, see: https://flutter.dev/to/review-gradle-config.
        // Pinned explicitly per DEC-001 §2 / ROADMAP D-020; do not revert to the inherited default.
        minSdk = 24
        targetSdk = flutter.targetSdkVersion
        // Uses the version code from pubspec.yaml. When using split APKs, 1000 * ABI_VERSION
        // is added automatically by Flutter. (https://developer.android.com/studio/build/configure-apk-splits#configure-APK-versions)
        // You can force using the value of versionCode by specifying the `-P force-version-code-ignoring-abi=true`
        // flag during build.
        versionCode = flutter.versionCode
        versionName = flutter.versionName
    }

    signingConfigs {
        create("release") {
            if (releaseKeystore != null) {
                storeFile = file(releaseKeystore)
                storePassword = signingSetting("storePassword", "MEALMATE_KEY_PASSWORD")
                keyAlias = signingSetting("keyAlias", "MEALMATE_KEY_ALIAS") ?: "mealmate"
                keyPassword = signingSetting("keyPassword", "MEALMATE_KEY_PASSWORD")
            }
        }
    }

    buildTypes {
        release {
            // Falls back to the debug key only on a clone with no key configured, so
            // `flutter build apk --release` still works out of the box. CI always configures
            // one, and the workflow's apksigner check fails the run if a build ever reaches
            // that fallback there. The build type is not debuggable either way.
            signingConfig = if (releaseKeystore != null) {
                signingConfigs.getByName("release")
            } else {
                signingConfigs.getByName("debug")
            }
        }
    }
}

kotlin {
    compilerOptions {
        jvmTarget = org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17
    }
}

flutter {
    source = "../.."
}
