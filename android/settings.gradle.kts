// The probe's shell (M0/W3, founding decision D4). One activity, one button, one text view.
pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}
dependencyResolutionManagement {
    repositories {
        google()
        mavenCentral()
    }
}
rootProject.name = "aurora-probe"
include(":app")
