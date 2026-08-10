---
name: flint-sdk-kotlin
description: >
  Install and use the Flint Realtime Fabric Kotlin/Android SDK (frf-kotlin). Covers Gradle
  integration, SpineClient setup, channel subscriptions, and event publishing for Android
  and JVM applications via JNI bridge.
version: '1.0.0'
license: MIT
metadata:
  author: Prometheus AGS
  version: '1.0.0'
  category: flint
  tags: [flint, realtime, kotlin, android, jvm, sdk, grpc, connect-rpc]
---

# flint-sdk-kotlin

Use the **Flint Realtime Fabric** Kotlin SDK (`frf-kotlin`) for Android and JVM applications.

## When to use

- Adding realtime event streaming to an Android app or Kotlin JVM service.
- Consuming FRF channels from Kotlin via JNI bridge to the Rust core.

## Requirements

- Kotlin 2.0+
- Android API 26+ (for Android targets)
- JDK 17+ (for JVM targets)

## Installation (Gradle)

Build the native library first:
```bash
# In flint-realtime-fabric repo
bash sdks/kotlin/build_jni.sh
```

Add to `settings.gradle.kts`:
```kotlin
include(":lib")
```

Add to your module `build.gradle.kts`:
```kotlin
dependencies {
    implementation(project(":lib"))
}
```

## Minimal example

```kotlin
import com.prometheusags.frf.SpineClient

val client = SpineClient("https://your-frf-gateway")

// Subscribe (coroutines)
launch {
    client.subscribe("my-channel").collect { event ->
        println(event)
        client.ack(event.cursor)
    }
}

// Publish
client.publish("my-channel", payload = "hello".encodeToByteArray())
```

## SDK source

Source code: `<flint-realtime-fabric>/sdks/kotlin/`. Resolve the repository root from the current workspace or `FLINT_REPO_ROOT`.
Root project: `frf-kotlin`
