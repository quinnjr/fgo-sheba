package io.sheba

import android.animation.ObjectAnimator
import android.animation.ValueAnimator
import android.app.*
import android.content.Context
import android.content.Intent
import android.graphics.PixelFormat
import android.os.Handler
import android.os.IBinder
import android.os.Looper
import android.util.Log
import android.view.*
import android.view.animation.AccelerateDecelerateInterpolator
import android.view.animation.AnimationUtils
import android.widget.FrameLayout
import android.widget.ImageView
import android.widget.LinearLayout
import android.widget.TextView
import androidx.core.app.NotificationCompat
import androidx.core.content.ContextCompat
import com.google.android.material.card.MaterialCardView
import kotlin.math.abs

/**
 * Service for managing the floating overlay UI with beautiful FGO-inspired design.
 */
class OverlayService : Service() {

    companion object {
        private const val TAG = "OverlayService"
        private const val NOTIFICATION_ID = 1002
        private const val CHANNEL_ID = "sheba_overlay_channel"
        private const val CLICK_THRESHOLD = 10
    }

    private var windowManager: WindowManager? = null
    private var overlayView: View? = null
    private var isPaused = false
    private var isMenuExpanded = false
    private var pulseAnimator: ValueAnimator? = null

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
        windowManager = getSystemService(Context.WINDOW_SERVICE) as WindowManager
        Log.i(TAG, "Overlay service created")
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        startForeground(NOTIFICATION_ID, createNotification())
        showOverlay()
        return START_STICKY
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onDestroy() {
        super.onDestroy()
        stopPulseAnimation()
        removeOverlay()
        Log.i(TAG, "Overlay service destroyed")
    }

    private fun createNotificationChannel() {
        val channel = NotificationChannel(
            CHANNEL_ID,
            "Sheba Overlay",
            NotificationManager.IMPORTANCE_LOW
        ).apply {
            description = "Floating overlay controls for Sheba automation"
            setShowBadge(false)
        }

        val notificationManager = getSystemService(NotificationManager::class.java)
        notificationManager.createNotificationChannel(channel)
    }

    private fun createNotification(): Notification {
        val pendingIntent = PendingIntent.getActivity(
            this,
            0,
            Intent(this, MainActivity::class.java),
            PendingIntent.FLAG_IMMUTABLE
        )

        val stopIntent = PendingIntent.getService(
            this,
            1,
            Intent(this, OverlayService::class.java).apply { action = "STOP" },
            PendingIntent.FLAG_IMMUTABLE
        )

        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("Sheba Active")
            .setContentText("AI automation is running")
            .setSmallIcon(R.drawable.ic_launcher_foreground)
            .setContentIntent(pendingIntent)
            .addAction(R.drawable.ic_stop, "Stop", stopIntent)
            .setOngoing(true)
            .setColor(ContextCompat.getColor(this, R.color.accent_gold))
            .build()
    }

    private fun showOverlay() {
        if (overlayView != null) return

        // Inflate the custom overlay layout
        val inflater = LayoutInflater.from(this)
        val view = inflater.inflate(R.layout.overlay_control, null)
        overlayView = view

        // Get references to views
        val fabContainer = view.findViewById<MaterialCardView>(R.id.fabContainer)
        val glowRing = view.findViewById<View>(R.id.glowRing)
        val expandedMenu = view.findViewById<LinearLayout>(R.id.expandedMenu)
        val btnPauseResume = view.findViewById<LinearLayout>(R.id.btnPauseResume)
        val btnStop = view.findViewById<LinearLayout>(R.id.btnStop)
        val ivPauseResume = view.findViewById<ImageView>(R.id.ivPauseResume)
        val tvPauseResume = view.findViewById<TextView>(R.id.tvPauseResume)

        // Layout parameters for the overlay
        val params = WindowManager.LayoutParams(
            WindowManager.LayoutParams.WRAP_CONTENT,
            WindowManager.LayoutParams.WRAP_CONTENT,
            WindowManager.LayoutParams.TYPE_APPLICATION_OVERLAY,
            WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE or
                WindowManager.LayoutParams.FLAG_NOT_TOUCH_MODAL or
                WindowManager.LayoutParams.FLAG_LAYOUT_NO_LIMITS,
            PixelFormat.TRANSLUCENT
        ).apply {
            gravity = Gravity.TOP or Gravity.END
            x = 20
            y = 200
        }

        // Make the FAB draggable with click detection
        var initialX = 0
        var initialY = 0
        var initialTouchX = 0f
        var initialTouchY = 0f
        var isDragging = false

        fabContainer.setOnTouchListener { _, event ->
            when (event.action) {
                MotionEvent.ACTION_DOWN -> {
                    initialX = params.x
                    initialY = params.y
                    initialTouchX = event.rawX
                    initialTouchY = event.rawY
                    isDragging = false
                    true
                }
                MotionEvent.ACTION_MOVE -> {
                    val deltaX = event.rawX - initialTouchX
                    val deltaY = event.rawY - initialTouchY

                    if (abs(deltaX) > CLICK_THRESHOLD || abs(deltaY) > CLICK_THRESHOLD) {
                        isDragging = true
                    }

                    if (isDragging) {
                        // For END gravity, x increases to the left
                        params.x = initialX - deltaX.toInt()
                        params.y = initialY + deltaY.toInt()
                        windowManager?.updateViewLayout(view, params)
                    }
                    true
                }
                MotionEvent.ACTION_UP -> {
                    if (!isDragging) {
                        // This was a tap, toggle menu
                        toggleMenu(expandedMenu, fabContainer)
                    }
                    true
                }
                else -> false
            }
        }

        // Pause/Resume button
        btnPauseResume.setOnClickListener {
            isPaused = !isPaused
            ShebaCore.setPaused(isPaused)

            if (isPaused) {
                tvPauseResume.text = "Resume"
                ivPauseResume.setImageResource(R.drawable.ic_play)
                stopPulseAnimation()
                glowRing.alpha = 0.2f
            } else {
                tvPauseResume.text = "Pause"
                ivPauseResume.setImageResource(R.drawable.ic_stop)
                startPulseAnimation(glowRing)
            }

            collapseMenu(expandedMenu, fabContainer)
        }

        // Stop button
        btnStop.setOnClickListener {
            collapseMenu(expandedMenu, fabContainer)

            // Small delay before stopping to show animation
            Handler(Looper.getMainLooper()).postDelayed({
                stopSelf()
                stopService(Intent(this@OverlayService, ScreenCaptureService::class.java))
            }, 200)
        }

        // Start the pulse animation
        startPulseAnimation(glowRing)

        windowManager?.addView(view, params)

        // Animate entrance
        view.alpha = 0f
        view.scaleX = 0.5f
        view.scaleY = 0.5f
        view.animate()
            .alpha(1f)
            .scaleX(1f)
            .scaleY(1f)
            .setDuration(300)
            .setInterpolator(AccelerateDecelerateInterpolator())
            .start()

        Log.i(TAG, "Overlay shown with new design")
    }

    private fun toggleMenu(menu: LinearLayout, fab: MaterialCardView) {
        if (isMenuExpanded) {
            collapseMenu(menu, fab)
        } else {
            expandMenu(menu, fab)
        }
    }

    private fun expandMenu(menu: LinearLayout, fab: MaterialCardView) {
        isMenuExpanded = true

        menu.visibility = View.VISIBLE
        menu.alpha = 0f
        menu.translationX = 50f
        menu.animate()
            .alpha(1f)
            .translationX(0f)
            .setDuration(200)
            .setInterpolator(AccelerateDecelerateInterpolator())
            .start()

        // Rotate the FAB slightly
        fab.animate()
            .rotation(45f)
            .setDuration(200)
            .start()
    }

    private fun collapseMenu(menu: LinearLayout, fab: MaterialCardView) {
        isMenuExpanded = false

        menu.animate()
            .alpha(0f)
            .translationX(50f)
            .setDuration(150)
            .withEndAction { menu.visibility = View.GONE }
            .start()

        fab.animate()
            .rotation(0f)
            .setDuration(200)
            .start()
    }

    private fun startPulseAnimation(view: View) {
        stopPulseAnimation()

        pulseAnimator = ValueAnimator.ofFloat(0.3f, 0.8f).apply {
            duration = 1200
            repeatCount = ValueAnimator.INFINITE
            repeatMode = ValueAnimator.REVERSE
            interpolator = AccelerateDecelerateInterpolator()
            addUpdateListener { animator ->
                view.alpha = animator.animatedValue as Float
            }
            start()
        }
    }

    private fun stopPulseAnimation() {
        pulseAnimator?.cancel()
        pulseAnimator = null
    }

    private fun removeOverlay() {
        overlayView?.let { view ->
            // Animate exit
            view.animate()
                .alpha(0f)
                .scaleX(0.5f)
                .scaleY(0.5f)
                .setDuration(200)
                .withEndAction {
                    try {
                        windowManager?.removeView(view)
                    } catch (e: Exception) {
                        Log.w(TAG, "Error removing overlay view", e)
                    }
                }
                .start()

            overlayView = null
        }
    }
}
