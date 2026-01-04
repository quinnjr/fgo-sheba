package io.sheba

import android.accessibilityservice.AccessibilityServiceInfo
import android.animation.AnimatorSet
import android.animation.ObjectAnimator
import android.animation.ValueAnimator
import android.app.ActivityManager
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.graphics.drawable.GradientDrawable
import android.media.projection.MediaProjectionManager
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.provider.Settings
import android.view.View
import android.view.accessibility.AccessibilityManager
import android.view.animation.AccelerateDecelerateInterpolator
import android.view.animation.AnimationUtils
import android.widget.Toast
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AlertDialog
import androidx.appcompat.app.AppCompatActivity
import androidx.core.content.ContextCompat
import androidx.core.view.WindowCompat
import io.sheba.databinding.ActivityMainBinding
import io.sheba.stealth.AntiDetectionService
import io.sheba.stealth.StealthManager

class MainActivity : AppCompatActivity() {

    private lateinit var binding: ActivityMainBinding
    private var isAutomationRunning = false

    // Session stats
    private var battleCount = 0
    private var npCount = 0
    private var runtimeSeconds = 0L
    private var runtimeHandler: Handler? = null
    private var runtimeRunnable: Runnable? = null

    // Stealth mode
    private var stealthReceiver: BroadcastReceiver? = null
    private var hasShownSecurityWarning = false

    private val mediaProjectionLauncher = registerForActivityResult(
        ActivityResultContracts.StartActivityForResult()
    ) { result ->
        if (result.resultCode == RESULT_OK && result.data != null) {
            // Start screen capture service
            val intent = Intent(this, ScreenCaptureService::class.java).apply {
                putExtra(ScreenCaptureService.EXTRA_RESULT_CODE, result.resultCode)
                putExtra(ScreenCaptureService.EXTRA_RESULT_DATA, result.data)
            }
            startForegroundService(intent)

            // Update UI
            isAutomationRunning = true
            startRuntimeCounter()
            updateUI()
            animateStatusChange(true)

            Toast.makeText(this, R.string.toast_automation_started, Toast.LENGTH_SHORT).show()
        } else {
            Toast.makeText(this, R.string.toast_screen_capture_denied, Toast.LENGTH_SHORT).show()
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Enable edge-to-edge
        WindowCompat.setDecorFitsSystemWindows(window, false)

        binding = ActivityMainBinding.inflate(layoutInflater)
        setContentView(binding.root)

        setupUI()
        setupStealthMode()
        startInitialAnimations()

        // Check and show security recommendations on first launch
        checkSecurityStatus()
    }

    override fun onResume() {
        super.onResume()
        updateUI()
        updateStealthStatus()
    }

    override fun onDestroy() {
        super.onDestroy()
        stopRuntimeCounter()
        unregisterStealthReceiver()
    }

    private fun setupStealthMode() {
        // Register receiver for stealth mode changes
        stealthReceiver = object : BroadcastReceiver() {
            override fun onReceive(context: Context?, intent: Intent?) {
                if (intent?.action == "io.sheba.UPDATE_RECENTS_VISIBILITY") {
                    val exclude = intent.getBooleanExtra("exclude", false)
                    setExcludeFromRecents(exclude)
                }
            }
        }

        val filter = IntentFilter("io.sheba.UPDATE_RECENTS_VISIBILITY")
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            registerReceiver(stealthReceiver, filter, RECEIVER_NOT_EXPORTED)
        } else {
            registerReceiver(stealthReceiver, filter)
        }
    }

    private fun unregisterStealthReceiver() {
        stealthReceiver?.let {
            try {
                unregisterReceiver(it)
            } catch (e: Exception) {
                // Ignore if already unregistered
            }
        }
    }

    private fun setExcludeFromRecents(exclude: Boolean) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.LOLLIPOP) {
            val am = getSystemService(Context.ACTIVITY_SERVICE) as ActivityManager
            val tasks = am.appTasks
            if (tasks.isNotEmpty()) {
                tasks[0].setExcludeFromRecents(exclude)
            }
        }
    }

    private fun updateStealthStatus() {
        // Check if FGO is running and update stealth mode accordingly
        if (StealthManager.isFGOInForeground(this)) {
            setExcludeFromRecents(true)
        }
    }

    private fun checkSecurityStatus() {
        if (hasShownSecurityWarning) return

        val recommendations = StealthManager.getSecurityRecommendations(this)
        val hasWarnings = recommendations.any { it.startsWith("⚠️") }

        if (hasWarnings) {
            hasShownSecurityWarning = true
            showSecurityDialog(recommendations)
        }
    }

    private fun showSecurityDialog(recommendations: List<String>) {
        val message = buildString {
            appendLine("Detection Risk Assessment:\n")
            recommendations.forEach { rec ->
                appendLine(rec)
                appendLine()
            }
            appendLine("These settings may increase the chance of detection by the game. Consider adjusting them for better safety.")
        }

        AlertDialog.Builder(this, R.style.Theme_Sheba_Dialog)
            .setTitle("Security Notice")
            .setMessage(message)
            .setPositiveButton("Understood") { dialog, _ -> dialog.dismiss() }
            .setNeutralButton("Don't show again") { dialog, _ ->
                getSharedPreferences("prefs", MODE_PRIVATE)
                    .edit()
                    .putBoolean("hide_security_warning", true)
                    .apply()
                dialog.dismiss()
            }
            .show()
    }

    private fun setupUI() {
        // Main automation toggle
        binding.btnToggleAutomation.setOnClickListener {
            if (isAutomationRunning) {
                stopAutomation()
            } else {
                startAutomation()
            }
        }

        // Permission items - clickable
        binding.permissionAccessibility.setOnClickListener {
            openAccessibilitySettings()
        }

        binding.permissionOverlay.setOnClickListener {
            openOverlaySettings()
        }

        // Settings button
        binding.btnSettings.setOnClickListener {
            // TODO: Open settings activity
            Toast.makeText(this, "Settings coming soon!", Toast.LENGTH_SHORT).show()
        }

        // Start pulse animation on logo ring
        startPulseAnimation()
    }

    private fun startInitialAnimations() {
        // Fade in the logo
        binding.ivLogo.alpha = 0f
        binding.ivLogo.animate()
            .alpha(1f)
            .setDuration(800)
            .setInterpolator(AccelerateDecelerateInterpolator())
            .start()

        // Slide up title
        binding.tvTitle.translationY = 50f
        binding.tvTitle.alpha = 0f
        binding.tvTitle.animate()
            .translationY(0f)
            .alpha(1f)
            .setStartDelay(200)
            .setDuration(600)
            .setInterpolator(AccelerateDecelerateInterpolator())
            .start()

        // Slide up subtitle
        binding.tvSubtitle.translationY = 30f
        binding.tvSubtitle.alpha = 0f
        binding.tvSubtitle.animate()
            .translationY(0f)
            .alpha(1f)
            .setStartDelay(350)
            .setDuration(500)
            .setInterpolator(AccelerateDecelerateInterpolator())
            .start()

        // Fade in status banner
        binding.statusBanner.alpha = 0f
        binding.statusBanner.animate()
            .alpha(1f)
            .setStartDelay(500)
            .setDuration(400)
            .start()
    }

    private fun startPulseAnimation() {
        val pulseAnim = AnimationUtils.loadAnimation(this, R.anim.pulse)
        binding.pulseRing.startAnimation(pulseAnim)
    }

    private fun animateStatusChange(running: Boolean) {
        val statusDot = binding.statusDot
        val statusGlow = binding.statusGlow

        // Change colors based on state
        val targetColor = if (running) {
            ContextCompat.getColor(this, R.color.status_success)
        } else {
            ContextCompat.getColor(this, R.color.status_error)
        }

        // Animate the status dot
        val scaleX = ObjectAnimator.ofFloat(statusDot, View.SCALE_X, 0.5f, 1.2f, 1f)
        val scaleY = ObjectAnimator.ofFloat(statusDot, View.SCALE_Y, 0.5f, 1.2f, 1f)

        AnimatorSet().apply {
            playTogether(scaleX, scaleY)
            duration = 400
            interpolator = AccelerateDecelerateInterpolator()
            start()
        }

        // Update background color
        (statusDot.background as? GradientDrawable)?.setColor(targetColor)
        (statusGlow.background as? GradientDrawable)?.setColor(targetColor)

        // Pulse the glow when running
        if (running) {
            val glowPulse = AnimationUtils.loadAnimation(this, R.anim.glow_pulse)
            statusGlow.startAnimation(glowPulse)
        } else {
            statusGlow.clearAnimation()
        }
    }

    private fun updateUI() {
        val accessibilityEnabled = isAccessibilityServiceEnabled()
        val overlayEnabled = Settings.canDrawOverlays(this)

        // Update accessibility permission status
        binding.tvAccessibilityStatus.text = if (accessibilityEnabled) {
            getString(R.string.permission_enabled)
        } else {
            getString(R.string.permission_disabled)
        }
        binding.ivAccessibilityStatus.setImageResource(
            if (accessibilityEnabled) R.drawable.ic_check_circle else R.drawable.ic_error_circle
        )

        // Update overlay permission status
        binding.tvOverlayStatus.text = if (overlayEnabled) {
            getString(R.string.permission_granted)
        } else {
            getString(R.string.permission_required)
        }
        binding.ivOverlayStatus.setImageResource(
            if (overlayEnabled) R.drawable.ic_check_circle else R.drawable.ic_error_circle
        )

        // Update main button
        val allPermissionsGranted = accessibilityEnabled && overlayEnabled
        binding.btnToggleAutomation.isEnabled = allPermissionsGranted

        if (isAutomationRunning) {
            binding.btnToggleAutomation.text = getString(R.string.stop_automation)
            binding.btnToggleAutomation.setBackgroundResource(R.drawable.bg_stop_button)
            binding.btnToggleAutomation.setTextColor(ContextCompat.getColor(this, R.color.white))
            binding.btnToggleAutomation.setIconResource(R.drawable.ic_stop)
            binding.btnToggleAutomation.setIconTintResource(R.color.white)
        } else {
            binding.btnToggleAutomation.text = getString(R.string.start_automation)
            binding.btnToggleAutomation.setBackgroundResource(R.drawable.bg_start_button)
            binding.btnToggleAutomation.setTextColor(ContextCompat.getColor(this, R.color.primary_dark))
            binding.btnToggleAutomation.setIconResource(R.drawable.ic_play)
            binding.btnToggleAutomation.setIconTintResource(R.color.primary_dark)
        }

        // Update status text
        binding.tvStatus.text = if (isAutomationRunning) {
            getString(R.string.status_running)
        } else {
            getString(R.string.status_idle)
        }

        // Update status indicator colors
        val statusColor = if (isAutomationRunning) {
            ContextCompat.getColor(this, R.color.status_success)
        } else {
            ContextCompat.getColor(this, R.color.status_error)
        }

        (binding.statusDot.background as? GradientDrawable)?.setColor(statusColor)
        (binding.statusGlow.background as? GradientDrawable)?.setColor(statusColor)

        // Update stats
        binding.tvBattleCount.text = battleCount.toString()
        binding.tvNPCount.text = npCount.toString()
    }

    private fun isAccessibilityServiceEnabled(): Boolean {
        val am = getSystemService(Context.ACCESSIBILITY_SERVICE) as AccessibilityManager
        val enabledServices = am.getEnabledAccessibilityServiceList(
            AccessibilityServiceInfo.FEEDBACK_ALL_MASK
        )

        return enabledServices.any {
            it.resolveInfo.serviceInfo.packageName == packageName &&
            it.resolveInfo.serviceInfo.name == ShebaAccessibilityService::class.java.name
        }
    }

    private fun openAccessibilitySettings() {
        val intent = Intent(Settings.ACTION_ACCESSIBILITY_SETTINGS)
        startActivity(intent)
    }

    private fun openOverlaySettings() {
        val intent = Intent(
            Settings.ACTION_MANAGE_OVERLAY_PERMISSION,
            Uri.parse("package:$packageName")
        )
        startActivity(intent)
    }

    private fun startAutomation() {
        if (!isAccessibilityServiceEnabled()) {
            Toast.makeText(this, R.string.toast_accessibility_required, Toast.LENGTH_SHORT).show()
            openAccessibilitySettings()
            return
        }

        if (!Settings.canDrawOverlays(this)) {
            Toast.makeText(this, R.string.toast_overlay_required, Toast.LENGTH_SHORT).show()
            openOverlaySettings()
            return
        }

        // Initialize Sheba core
        val initialized = ShebaCore.init(null)
        if (!initialized) {
            Toast.makeText(this, R.string.toast_init_failed, Toast.LENGTH_SHORT).show()
            return
        }

        // Start anti-detection service first
        startAntiDetectionService()

        // Request screen capture permission
        val mediaProjectionManager = getSystemService(Context.MEDIA_PROJECTION_SERVICE) as MediaProjectionManager
        val captureIntent = mediaProjectionManager.createScreenCaptureIntent()
        mediaProjectionLauncher.launch(captureIntent)
    }

    private fun startAntiDetectionService() {
        val intent = Intent(this, AntiDetectionService::class.java)
        startForegroundService(intent)
    }

    private fun stopAntiDetectionService() {
        val intent = Intent(this, AntiDetectionService::class.java)
        stopService(intent)
    }

    private fun stopAutomation() {
        // Stop screen capture service
        val intent = Intent(this, ScreenCaptureService::class.java)
        stopService(intent)

        // Stop overlay service
        val overlayIntent = Intent(this, OverlayService::class.java)
        stopService(overlayIntent)

        // Stop anti-detection service
        stopAntiDetectionService()

        // Restore visibility in recents
        setExcludeFromRecents(false)

        // Update state
        isAutomationRunning = false
        ShebaCore.setPaused(true)
        stopRuntimeCounter()
        updateUI()
        animateStatusChange(false)

        Toast.makeText(this, R.string.toast_automation_stopped, Toast.LENGTH_SHORT).show()
    }

    private fun startRuntimeCounter() {
        runtimeHandler = Handler(Looper.getMainLooper())
        runtimeRunnable = object : Runnable {
            override fun run() {
                runtimeSeconds++
                updateRuntimeDisplay()
                runtimeHandler?.postDelayed(this, 1000)
            }
        }
        runtimeHandler?.post(runtimeRunnable!!)
    }

    private fun stopRuntimeCounter() {
        runtimeRunnable?.let { runtimeHandler?.removeCallbacks(it) }
        runtimeHandler = null
        runtimeRunnable = null
    }

    private fun updateRuntimeDisplay() {
        val hours = runtimeSeconds / 3600
        val minutes = (runtimeSeconds % 3600) / 60
        val seconds = runtimeSeconds % 60

        binding.tvRuntime.text = if (hours > 0) {
            String.format("%d:%02d:%02d", hours, minutes, seconds)
        } else {
            String.format("%02d:%02d", minutes, seconds)
        }
    }

    // Called from automation service to update stats
    fun incrementBattleCount() {
        battleCount++
        runOnUiThread { binding.tvBattleCount.text = battleCount.toString() }
    }

    fun incrementNPCount() {
        npCount++
        runOnUiThread { binding.tvNPCount.text = npCount.toString() }
    }
}
