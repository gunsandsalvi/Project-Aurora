package com.aurora.probe

import android.app.Activity
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.os.Bundle
import android.view.View
import android.view.WindowManager
import android.widget.Button
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import android.widget.Toast
import java.io.BufferedReader
import java.io.File
import java.io.InputStreamReader
import kotlin.concurrent.thread

/**
 * The whole shell (M0/W3, founding decision D4). One Run button, one scrolling text view, one Copy
 * button. It measures nothing itself: it runs the native probe as a subprocess and shows its stdout,
 * which is one JSON document conforming to `aurora.probe/1`.
 *
 * WHY A SUBPROCESS AND NOT JNI. The probe is a PIE executable packaged as
 * `lib/arm64-v8a/libaurora_probe.so`; Android extracts anything matching `lib*.so` with the execute
 * bit set. A JNI entry point would need `unsafe` on the Rust side and would put the measurement
 * behind a foreign-function boundary whose cost is not the thing being measured. `forbid(unsafe_code)`
 * survives into the delivered artifact this way.
 *
 * WHY NO FOREGROUND SERVICE. The plan called for one so a long run is not killed. The run is a child
 * process of this app and the screen is held on for its duration, which keeps the app foreground and
 * the child alive; a service would keep the *app* alive while backgrounded, which is not the case
 * being defended against. If a soak is ever actually killed, that is the evidence for adding one.
 *
 * WHAT `largeHeap` DOES HERE: nothing. It raises the JVM heap ceiling, and every byte the probe
 * allocates is in another process. It is declared because the plan declared it, and this comment is
 * the correction — the allocation ceiling the probe reports is a kernel and cgroup limit, not a
 * Dalvik one, which is the reason the measurement is worth taking at all.
 */
class MainActivity : Activity() {

    private lateinit var output: TextView
    private lateinit var run: Button
    private lateinit var copy: Button
    private var result: String = ""

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        output = TextView(this).apply {
            textSize = 11f
            typeface = android.graphics.Typeface.MONOSPACE
            setPadding(24, 24, 24, 24)
            setTextIsSelectable(true)
            text = "Tap Run.\n\n" +
                "The full run takes about twenty minutes: fifteen of them are the thermal soak, " +
                "which is the point — every other figure here is a burst, and the tick budget is " +
                "the sustained one. Keep the screen on and the app in front.\n\n" +
                "When it finishes, tap Copy and paste the JSON back."
        }

        run = Button(this).apply {
            text = "Run"
            setOnClickListener { start() }
        }
        copy = Button(this).apply {
            text = "Copy"
            isEnabled = false
            setOnClickListener {
                val cb = getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
                cb.setPrimaryClip(ClipData.newPlainText("aurora.probe/1", result))
                Toast.makeText(this@MainActivity, "Copied", Toast.LENGTH_SHORT).show()
            }
        }

        val buttons = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            addView(run, LinearLayout.LayoutParams(0, -2, 1f))
            addView(copy, LinearLayout.LayoutParams(0, -2, 1f))
        }
        val scroll = ScrollView(this).apply { addView(output) }
        val root = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            addView(buttons, LinearLayout.LayoutParams(-1, -2))
            addView(scroll, LinearLayout.LayoutParams(-1, 0, 1f))
        }
        setContentView(root)
    }

    private fun start() {
        run.isEnabled = false
        copy.isEnabled = false
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        output.text = "Running. Do not lock the screen.\n"

        thread {
            val text = try {
                execute()
            } catch (t: Throwable) {
                "FAILED: ${t.javaClass.simpleName}: ${t.message}"
            }
            runOnUiThread {
                result = text
                output.text = text
                run.isEnabled = true
                copy.isEnabled = text.trimStart().startsWith("{")
                window.clearFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
            }
        }
    }

    /** Run the native probe and return everything it printed. */
    private fun execute(): String {
        val binary = File(applicationInfo.nativeLibraryDir, "libaurora_probe.so")
        if (!binary.exists()) {
            return "FAILED: ${binary.path} is not in the APK. The native build did not run."
        }
        val model = "${android.os.Build.MANUFACTURER} ${android.os.Build.MODEL} " +
            "(android ${android.os.Build.VERSION.RELEASE}, sdk ${android.os.Build.VERSION.SDK_INT})"
        val process = ProcessBuilder(
            binary.path, "--device", "--device-name", model
        )
            .directory(filesDir)
            .redirectErrorStream(true)
            .start()

        val text = BufferedReader(InputStreamReader(process.inputStream)).use { it.readText() }
        val code = process.waitFor()
        return if (code == 0) text else "FAILED: exit $code\n\n$text"
    }
}
