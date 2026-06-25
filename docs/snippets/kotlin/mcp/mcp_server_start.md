```kotlin title="Kotlin"
import io.xberg.*
import java.util.Optional

fun main() {
    val process = ProcessBuilder("xberg", "mcp")
        .inheritIO()
        .start()
    process.waitFor()
}
```
