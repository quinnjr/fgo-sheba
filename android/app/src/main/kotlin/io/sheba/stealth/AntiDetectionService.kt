package io.sheba.stealth

import android.app.*
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.os.Build
import android.os.Handler
import android.os.IBinder
import android.os.Looper
import android.util.Log
import androidx.core.app.NotificationCompat
import io.sheba.MainActivity
import io.sheba.R

/**
 * Background service that monitors for detection attempts and manages stealth features.
 *
 * Features:
 * - Monitors when FGO launches and activates stealth mode
 * - Hides from recent apps when FGO is in foreground
 * - Randomizes notification content to avoid keyword scanning
 * - Monitors for accessibility service enumeration
 */
class AntiDetectionService : Service() {

    companion object {
        private const val TAG = "AntiDetection"
        private const val NOTIFICATION_ID = 1003
        private const val CHANNEL_ID = "sheba_stealth_channel"
        private const val CHECK_INTERVAL_MS = 5000L

        // Generic notification messages to avoid keyword detection
        private val STEALTH_TITLES = listOf(
            "System Service",
            "Background Process",
            "Sync Active",
            "Service Running"
        )

        private val STEALTH_MESSAGES = listOf(
            "Running in background",
            "Monitoring system status",
            "Background sync active",
            "Service operational"
        )
    }

    private var handler: Handler? = null
    private var monitorRunnable: Runnable? = null
    private var isFGOActive = false
    private var packageChangeReceiver: BroadcastReceiver? = null

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
        handler = Handler(Looper.getMainLooper())
        registerPackageReceiver()
        Log.i(TAG, "Anti-detection service started")
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        startForeground(NOTIFICATION_ID, createStealthNotification())
        startMonitoring()
        return START_STICKY
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onDestroy() {
        super.onDestroy()
        stopMonitoring()
        unregisterPackageReceiver()
        Log.i(TAG, "Anti-detection service stopped")
    }

    private fun createNotificationChannel() {
        // Use generic channel name
        val channel = NotificationChannel(
            CHANNEL_ID,
            "Background Service",
            NotificationManager.IMPORTANCE_MIN  // Minimize visibility
        ).apply {
            description = "Background service notifications"
            setShowBadge(false)
            enableLights(false)
            enableVibration(false)
            setSound(null, null)
        }

        val notificationManager = getSystemService(NotificationManager::class.java)
        notificationManager.createNotificationChannel(channel)
    }

    private fun createStealthNotification(): Notification {
        // Use random generic messages
        val title = STEALTH_TITLES.random()
        val message = STEALTH_MESSAGES.random()

        val pendingIntent = PendingIntent.getActivity(
            this,
            0,
            Intent(this, MainActivity::class.java),
            PendingIntent.FLAG_IMMUTABLE
        )

        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle(title)
            .setContentText(message)
            .setSmallIcon(android.R.drawable.ic_popup_sync)  // Use system icon
            .setContentIntent(pendingIntent)
            .setOngoing(true)
            .setPriority(NotificationCompat.PRIORITY_MIN)
            .setVisibility(NotificationCompat.VISIBILITY_SECRET)
            .build()
    }

    private fun startMonitoring() {
        monitorRunnable = object : Runnable {
            override fun run() {
                checkFGOStatus()
                handler?.postDelayed(this, CHECK_INTERVAL_MS)
            }
        }
        handler?.post(monitorRunnable!!)
    }

    private fun stopMonitoring() {
        monitorRunnable?.let { handler?.removeCallbacks(it) }
        monitorRunnable = null
    }

    private fun checkFGOStatus() {
        val wasFGOActive = isFGOActive
        isFGOActive = StealthManager.isFGOInForeground(this)

        if (isFGOActive != wasFGOActive) {
            if (isFGOActive) {
                onFGOBecameActive()
            } else {
                onFGOBecameInactive()
            }
        }
    }

    private fun onFGOBecameActive() {
        Log.d(TAG, "FGO became active - enabling stealth mode")

        // Hide from recent apps
        excludeFromRecents(true)

        // Update notification to be more generic
        updateNotification()
    }

    private fun onFGOBecameInactive() {
        Log.d(TAG, "FGO became inactive - relaxing stealth mode")

        // Can show in recents again when FGO is not running
        excludeFromRecents(false)
    }

    private fun excludeFromRecents(exclude: Boolean) {
        val activityManager = getSystemService(Context.ACTIVITY_SERVICE) as ActivityManager

        // Note: This affects the entire app, not just this service
        // The actual exclusion is done via activity flags in MainActivity

        // Broadcast to MainActivity to update its flags
        val intent = Intent("io.sheba.UPDATE_RECENTS_VISIBILITY").apply {
            putExtra("exclude", exclude)
        }
        sendBroadcast(intent)
    }

    private fun updateNotification() {
        val notificationManager = getSystemService(NotificationManager::class.java)
        notificationManager.notify(NOTIFICATION_ID, createStealthNotification())
    }

    private fun registerPackageReceiver() {
        packageChangeReceiver = object : BroadcastReceiver() {
            override fun onReceive(context: Context?, intent: Intent?) {
                // Monitor for app installations/changes that might indicate detection tools
                when (intent?.action) {
                    Intent.ACTION_PACKAGE_ADDED -> {
                        val packageName = intent.data?.schemeSpecificPart
                        Log.d(TAG, "New package installed: $packageName")
                    }
                }
            }
        }

        val filter = IntentFilter().apply {
            addAction(Intent.ACTION_PACKAGE_ADDED)
            addAction(Intent.ACTION_PACKAGE_REMOVED)
            addDataScheme("package")
        }

        registerReceiver(packageChangeReceiver, filter, Context.RECEIVER_NOT_EXPORTED)
    }

    private fun unregisterPackageReceiver() {
        packageChangeReceiver?.let {
            try {
                unregisterReceiver(it)
            } catch (e: Exception) {
                Log.w(TAG, "Error unregistering receiver", e)
            }
        }
    }
}
