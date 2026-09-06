plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "com.aurora.probe"
    compileSdk = 35

    defaultConfig {
        applicationId = "com.aurora.probe"
        minSdk = 26
        targetSdk = 35
        versionCode = 1
        versionName = "0.1"
        ndk { abiFilters += "arm64-v8a" }
    }

    // The measurement is a native PIE executable packaged as `lib/arm64-v8a/libaurora_probe.so`.
    // Android extracts anything matching `lib*.so` from that directory with the execute bit set, so
    // the activity runs it as a subprocess — but ONLY when the libraries are extracted rather than
    // loaded from inside the APK. Compression must be off for the same reason.
    packaging {
        jniLibs {
            useLegacyPackaging = true
        }
    }

    buildTypes {
        // Debug only, and deliberately: a debug APK is self-signed by the SDK's own key, which is
        // what makes it installable from a Release link with no keystore in this repository. The
        // measurement is a native binary compiled --release, so the Kotlin build type costs nothing.
        getByName("debug") {
            isMinifyEnabled = false
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions { jvmTarget = "17" }
}

dependencies {}
