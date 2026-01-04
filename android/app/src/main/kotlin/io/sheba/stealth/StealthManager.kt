package io.sheba.stealth

import android.app.ActivityManager
import android.content.Context
import android.content.pm.ApplicationInfo
import android.content.pm.PackageManager
import android.os.Build
import android.provider.Settings
import android.util.Log
import java.io.File
import kotlin.random.Random

/**
 * Manages stealth features to avoid detection by the target game.
 *
 * Detection vectors we protect against:
 * 1. Package name scanning - Use innocuous package name
 * 2. Accessibility service enumeration - Generic service name
 * 3. Overlay detection - Minimize overlay footprint
 * 4. Running process list - Hide from task list
 * 5. Timing analysis - Randomize action delays
 * 6. Root/Magisk detection - Don't require root
 * 7. USB debugging detection - Warn user
 */
object StealthManager {

    private const val TAG = "StealthMgr"

    // Known FGO package names to detect when game is active
    private val FGO_PACKAGES = setOf(
        "com.aniplex.fategrandorder",      // JP
        "com.aniplex.fategrandorder.en",   // NA
        "com.aniplex.fategrandorder.kr",   // KR
        "com.aniplex.fategrandorder.tw",   // TW
    )

    // Suspicious package keywords that games scan for
    private val SUSPICIOUS_KEYWORDS = listOf(
        "auto", "bot", "cheat", "hack", "macro", "script",
        "automata", "farmer", "clicker", "repeat", "touch"
    )

    /**
     * Check if FGO is currently running in foreground
     */
    fun isFGOInForeground(context: Context): Boolean {
        val activityManager = context.getSystemService(Context.ACTIVITY_SERVICE) as ActivityManager
        val runningApps = activityManager.runningAppProcesses ?: return false

        return runningApps.any { processInfo ->
            processInfo.importance == ActivityManager.RunningAppProcessInfo.IMPORTANCE_FOREGROUND &&
            FGO_PACKAGES.any { pkg -> processInfo.processName.contains(pkg) }
        }
    }

    /**
     * Check if FGO is installed
     */
    fun isFGOInstalled(context: Context): String? {
        val pm = context.packageManager
        for (pkg in FGO_PACKAGES) {
            try {
                pm.getPackageInfo(pkg, 0)
                return pkg
            } catch (e: PackageManager.NameNotFoundException) {
                continue
            }
        }
        return null
    }

    /**
     * Check if our package name could be detected as suspicious
     */
    fun isPackageNameSafe(context: Context): Boolean {
        val packageName = context.packageName.lowercase()
        return SUSPICIOUS_KEYWORDS.none { keyword -> packageName.contains(keyword) }
    }

    /**
     * Check if USB debugging is enabled (games often check this)
     */
    fun isUSBDebuggingEnabled(context: Context): Boolean {
        return Settings.Global.getInt(
            context.contentResolver,
            Settings.Global.ADB_ENABLED, 0
        ) == 1
    }

    /**
     * Check if developer options are enabled
     */
    fun isDeveloperOptionsEnabled(context: Context): Boolean {
        return Settings.Global.getInt(
            context.contentResolver,
            Settings.Global.DEVELOPMENT_SETTINGS_ENABLED, 0
        ) == 1
    }

    /**
     * Generate random human-like delay for actions
     * This helps avoid pattern detection
     */
    fun getHumanizedDelay(baseDelayMs: Long, variancePercent: Int = 30): Long {
        val variance = (baseDelayMs * variancePercent / 100)
        val randomOffset = Random.nextLong(-variance, variance + 1)
        return (baseDelayMs + randomOffset).coerceAtLeast(50)
    }

    /**
     * Generate random tap offset to simulate human imprecision
     */
    fun getHumanizedTapOffset(maxOffsetPx: Int = 5): Pair<Int, Int> {
        return Pair(
            Random.nextInt(-maxOffsetPx, maxOffsetPx + 1),
            Random.nextInt(-maxOffsetPx, maxOffsetPx + 1)
        )
    }

    /**
     * Check for common root indicators
     * We don't require root, but warn if detected as games check for it
     */
    fun checkRootIndicators(): List<String> {
        val indicators = mutableListOf<String>()

        // Check for su binary
        val suPaths = listOf(
            "/system/bin/su",
            "/system/xbin/su",
            "/sbin/su",
            "/data/local/xbin/su",
            "/data/local/bin/su",
            "/system/sd/xbin/su",
            "/system/bin/failsafe/su",
            "/data/local/su"
        )

        for (path in suPaths) {
            if (File(path).exists()) {
                indicators.add("su binary found: $path")
            }
        }

        // Check for Magisk
        if (File("/sbin/.magisk").exists() ||
            File("/data/adb/magisk").exists()) {
            indicators.add("Magisk detected")
        }

        // Check ro.debuggable
        try {
            val process = Runtime.getRuntime().exec("getprop ro.debuggable")
            val result = process.inputStream.bufferedReader().readText().trim()
            if (result == "1") {
                indicators.add("ro.debuggable=1")
            }
        } catch (e: Exception) {
            // Ignore
        }

        return indicators
    }

    /**
     * Get security recommendations for the user
     */
    fun getSecurityRecommendations(context: Context): List<String> {
        val recommendations = mutableListOf<String>()

        if (isUSBDebuggingEnabled(context)) {
            recommendations.add("⚠️ USB Debugging is enabled. Consider disabling it while playing FGO.")
        }

        if (isDeveloperOptionsEnabled(context)) {
            recommendations.add("⚠️ Developer Options are enabled. Some games detect this.")
        }

        val rootIndicators = checkRootIndicators()
        if (rootIndicators.isNotEmpty()) {
            recommendations.add("⚠️ Root indicators detected. Use Magisk Hide or similar if needed.")
        }

        if (!isPackageNameSafe(context)) {
            recommendations.add("⚠️ Package name contains suspicious keywords.")
        }

        if (recommendations.isEmpty()) {
            recommendations.add("✓ No obvious detection risks found.")
        }

        return recommendations
    }

    /**
     * Log stealth status (for debugging, disable in release)
     */
    fun logStealthStatus(context: Context) {
        if (!BuildConfig.DEBUG) return

        Log.d(TAG, "=== Stealth Status ===")
        Log.d(TAG, "FGO installed: ${isFGOInstalled(context)}")
        Log.d(TAG, "FGO foreground: ${isFGOInForeground(context)}")
        Log.d(TAG, "Package safe: ${isPackageNameSafe(context)}")
        Log.d(TAG, "USB debug: ${isUSBDebuggingEnabled(context)}")
        Log.d(TAG, "Dev options: ${isDeveloperOptionsEnabled(context)}")
        Log.d(TAG, "Root indicators: ${checkRootIndicators()}")
    }
}

/**
 * Build config placeholder - will be generated by Gradle
 */
object BuildConfig {
    const val DEBUG = true
}
