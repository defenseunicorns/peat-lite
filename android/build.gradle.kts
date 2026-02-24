// Copyright (c) 2025-2026 (r)evolve - Revolve Team LLC
// SPDX-License-Identifier: Apache-2.0

plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
    id("maven-publish")
    id("signing")
}

group = "com.defenseunicorns"
version = "0.0.4"  // Signed CannedMessage support

android {
    namespace = "com.defenseunicorns.eche.lite"
    compileSdk = 34

    defaultConfig {
        minSdk = 26  // Wear OS 3 minimum
        targetSdk = 34

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        consumerProguardFiles("consumer-rules.pro")

        ndk {
            abiFilters += listOf("arm64-v8a", "armeabi-v7a", "x86_64")
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }

    sourceSets {
        getByName("main") {
            jniLibs.srcDirs("src/main/jniLibs")
        }
    }

    publishing {
        singleVariant("release") {
            withSourcesJar()
        }
    }
}

dependencies {
    // JNA for UniFFI FFI calls
    implementation("net.java.dev.jna:jna:5.14.0@aar")

    // Kotlin standard library
    implementation("org.jetbrains.kotlin:kotlin-stdlib:1.9.20")

    // Testing
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.1.5")
}

// Task to build native libraries using Cargo
tasks.register<Exec>("buildNativeLibs") {
    description = "Build native Rust libraries for Android"
    group = "build"

    val hiveLiteRoot = rootProject.projectDir.parentFile
    workingDir = hiveLiteRoot

    val ndkPath = System.getenv("ANDROID_NDK_HOME")
        ?: System.getenv("NDK_HOME")
        ?: "${System.getenv("ANDROID_HOME")}/ndk/27.0.12077973"

    environment("ANDROID_NDK_HOME", ndkPath)
    environment("PATH", "$ndkPath/toolchains/llvm/prebuilt/linux-x86_64/bin:${System.getenv("PATH")}")

    commandLine("bash", "-c", """
        set -e
        echo "Building eche-lite-android native libraries from: $(pwd)"
        cd android-ffi

        # Build for arm64-v8a (modern Android devices)
        echo "Building for aarch64-linux-android (arm64-v8a)..."
        cargo build --release --target aarch64-linux-android
        mkdir -p ../android/src/main/jniLibs/arm64-v8a
        cp ../target/aarch64-linux-android/release/libeche_lite_android.so ../android/src/main/jniLibs/arm64-v8a/

        # Build for armeabi-v7a (older devices)
        echo "Building for armv7-linux-androideabi (armeabi-v7a)..."
        cargo build --release --target armv7-linux-androideabi
        mkdir -p ../android/src/main/jniLibs/armeabi-v7a
        cp ../target/armv7-linux-androideabi/release/libeche_lite_android.so ../android/src/main/jniLibs/armeabi-v7a/

        # Build for x86_64 (emulators)
        echo "Building for x86_64-linux-android (x86_64)..."
        cargo build --release --target x86_64-linux-android
        mkdir -p ../android/src/main/jniLibs/x86_64
        cp ../target/x86_64-linux-android/release/libeche_lite_android.so ../android/src/main/jniLibs/x86_64/

        echo ""
        echo "Native libraries built successfully!"
    """.trimIndent())
}

// Task to generate Kotlin bindings from UniFFI
tasks.register<Exec>("generateBindings") {
    description = "Generate Kotlin bindings from UniFFI"
    group = "build"

    dependsOn("buildNativeLibs")

    val hiveLiteRoot = rootProject.projectDir.parentFile
    workingDir = hiveLiteRoot

    commandLine("bash", "-c", """
        set -e
        echo "Generating Kotlin bindings..."
        cd android-ffi

        # Generate bindings using uniffi-bindgen
        cargo run --bin uniffi-bindgen generate \
            --library ../target/aarch64-linux-android/release/libeche_lite_android.so \
            --language kotlin \
            --out-dir ../android/src/main/java

        echo "Kotlin bindings generated in android/src/main/java/"
    """.trimIndent())
}

// Task to clean native libraries
tasks.register<Delete>("cleanNativeLibs") {
    description = "Clean native Rust libraries"
    group = "build"

    delete(
        "src/main/jniLibs/arm64-v8a/libeche_lite_android.so",
        "src/main/jniLibs/armeabi-v7a/libeche_lite_android.so",
        "src/main/jniLibs/x86_64/libeche_lite_android.so"
    )
}

// Combined task: build native libs + generate bindings + assemble AAR
tasks.register("buildAar") {
    description = "Build native libraries, generate bindings, and assemble AAR"
    group = "build"

    dependsOn("generateBindings")
    finalizedBy("assembleRelease")
}

// Publishing configuration
afterEvaluate {
    publishing {
        publications {
            register<MavenPublication>("release") {
                groupId = "com.defenseunicorns"
                artifactId = "eche-lite"
                version = project.version.toString()

                from(components["release"])

                pom {
                    name.set("Eche Lite Android")
                    description.set("Lightweight CRDT primitives for Eche Protocol - Android library by Defense Unicorns")
                    url.set("https://github.com/defenseunicorns/eche-lite")

                    licenses {
                        license {
                            name.set("Apache License 2.0")
                            url.set("https://www.apache.org/licenses/LICENSE-2.0")
                        }
                    }

                    developers {
                        developer {
                            id.set("revolve")
                            name.set("Revolve Team")
                            email.set("team@revolve.tech")
                        }
                    }

                    scm {
                        connection.set("scm:git:git://github.com/defenseunicorns/eche-lite.git")
                        developerConnection.set("scm:git:ssh://github.com/defenseunicorns/eche-lite.git")
                        url.set("https://github.com/defenseunicorns/eche-lite")
                    }
                }
            }
        }

        repositories {
            maven {
                name = "local"
                url = uri(layout.buildDirectory.dir("repo"))
            }
        }
    }

    signing {
        useGpgCmd()
        sign(publishing.publications["release"])
    }
}

// Task to create Maven Central bundle ZIP
tasks.register<Zip>("createMavenCentralBundle") {
    description = "Create ZIP bundle for Maven Central upload"
    group = "publishing"

    dependsOn("publishReleasePublicationToLocalRepository")

    from(layout.buildDirectory.dir("repo"))
    archiveFileName.set("eche-lite-${project.version}-bundle.zip")
    destinationDirectory.set(layout.buildDirectory.dir("bundle"))
}

// Task to publish to Maven Central via Central Portal API
tasks.register<Exec>("publishToMavenCentral") {
    description = "Upload bundle to Maven Central via Sonatype Central Portal"
    group = "publishing"

    dependsOn("createMavenCentralBundle")

    val bundleFile = layout.buildDirectory.file("bundle/eche-lite-${project.version}-bundle.zip")
    val username = project.findProperty("sonatypeUsername") as String? ?: System.getenv("SONATYPE_USERNAME") ?: ""
    val password = project.findProperty("sonatypePassword") as String? ?: System.getenv("SONATYPE_PASSWORD") ?: ""

    doFirst {
        if (username.isEmpty() || password.isEmpty()) {
            throw GradleException("Sonatype credentials not configured. Set SONATYPE_USERNAME and SONATYPE_PASSWORD environment variables.")
        }
    }

    commandLine("bash", "-c", """
        curl --fail-with-body \
            -u "$username:$password" \
            -F "bundle=@${bundleFile.get().asFile.absolutePath}" \
            "https://central.sonatype.com/api/v1/publisher/upload?publishingType=AUTOMATIC"
    """.trimIndent())
}
