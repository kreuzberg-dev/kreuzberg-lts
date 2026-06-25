```kotlin title="Kotlin"
import io.xberg.*
import io.xberg.kt.Xberg
import kotlinx.coroutines.runBlocking
import java.nio.file.Files
import java.nio.file.Paths

fun main() = runBlocking {
    val content = Files.readAllBytes(Paths.get("document.pdf"))
    val config = ExtractionConfig.builder().build()
    val result = Xberg.extractBytes(content, "application/pdf", config)

    println(result.content())
    println("Tables: ${result.tables()?.size ?: 0}")
}
```
