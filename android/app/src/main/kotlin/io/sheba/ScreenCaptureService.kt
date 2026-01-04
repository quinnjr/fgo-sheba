package io.sheba

import android.app.*
import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import android.graphics.PixelFormat
import android.hardware.display.DisplayManager
import android.hardware.display.VirtualDisplay
import android.media.Image
import android.media.ImageReader
import android.media.projection.MediaProjection
import android.media.projection.MediaProjectionManager
import android.os.Build
import android.os.IBinder
import android.util.DisplayMetrics
import android.util.Log
import android.view.WindowManager
import androidx.core.app.NotificationCompat
import kotlinx.coroutines.*
import kotlin.coroutines.coroutineContext
import java.nio.ByteBuffer

/**
 * Service for capturing screen content using MediaProjection.
 */
class ScreenCaptureService : Service() {

    companion object {
        private const val TAG = "ScreenCapture"
        const val EXTRA_RESULT_CODE = "result_code"
        const val EXTRA_RESULT_DATA = "result_data"
        private const val NOTIFICATION_ID = 1001
        private const val CHANNEL_ID = "sheba_capture_channel"
    }

    private var mediaProjection: MediaProjection? = null
    private var virtualDisplay: VirtualDisplay? = null
    private var imageReader: ImageReader? = null

    private val serviceScope = CoroutineScope(Dispatchers.Default + SupervisorJob())
    private var captureJob: Job? = null

    private var screenWidth = 0
    private var screenHeight = 0
    private var screenDensity = 0

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()

        // Get screen metrics
        val wm = getSystemService(Context.WINDOW_SERVICE) as WindowManager
        val metrics = DisplayMetrics()
        wm.defaultDisplay.getRealMetrics(metrics)

        screenWidth = metrics.widthPixels
        screenHeight = metrics.heightPixels
        screenDensity = metrics.densityDpi

        Log.i(TAG, "Screen capture service created: ${screenWidth}x${screenHeight}")
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        intent?.let {
            val resultCode = it.getIntExtra(EXTRA_RESULT_CODE, Activity.RESULT_CANCELED)
            val resultData: Intent? = it.getParcelableExtra(EXTRA_RESULT_DATA)

            if (resultCode == Activity.RESULT_OK && resultData != null) {
                startForeground(NOTIFICATION_ID, createNotification())
                startCapture(resultCode, resultData)
            }
        }

        return START_STICKY
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onDestroy() {
        super.onDestroy()
        stopCapture()
        serviceScope.cancel()
        Log.i(TAG, "Screen capture service destroyed")
    }

    private fun createNotificationChannel() {
        val channel = NotificationChannel(
            CHANNEL_ID,
            getString(R.string.notification_channel_name),
            NotificationManager.IMPORTANCE_LOW
        ).apply {
            description = getString(R.string.notification_channel_description)
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

        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle(getString(R.string.notification_title))
            .setContentText(getString(R.string.notification_text))
            .setSmallIcon(R.drawable.ic_launcher_foreground)
            .setContentIntent(pendingIntent)
            .setOngoing(true)
            .build()
    }

    private fun startCapture(resultCode: Int, resultData: Intent) {
        val mediaProjectionManager = getSystemService(Context.MEDIA_PROJECTION_SERVICE) as MediaProjectionManager

        mediaProjection = mediaProjectionManager.getMediaProjection(resultCode, resultData)
        mediaProjection?.registerCallback(object : MediaProjection.Callback() {
            override fun onStop() {
                Log.i(TAG, "MediaProjection stopped")
                stopCapture()
            }
        }, null)

        // Create ImageReader
        imageReader = ImageReader.newInstance(
            screenWidth,
            screenHeight,
            PixelFormat.RGBA_8888,
            2
        )

        // Create VirtualDisplay
        virtualDisplay = mediaProjection?.createVirtualDisplay(
            "ShebaCapture",
            screenWidth,
            screenHeight,
            screenDensity,
            DisplayManager.VIRTUAL_DISPLAY_FLAG_AUTO_MIRROR,
            imageReader?.surface,
            null,
            null
        )

        // Start capture loop
        captureJob = serviceScope.launch {
            captureLoop()
        }

        // Start overlay service
        val overlayIntent = Intent(this, OverlayService::class.java)
        startForegroundService(overlayIntent)

        Log.i(TAG, "Screen capture started")
    }

    private fun stopCapture() {
        captureJob?.cancel()
        captureJob = null

        virtualDisplay?.release()
        virtualDisplay = null

        imageReader?.close()
        imageReader = null

        mediaProjection?.stop()
        mediaProjection = null
    }

    private suspend fun captureLoop() {
        while (coroutineContext.isActive) {
            try {
                val image = imageReader?.acquireLatestImage()
                image?.let {
                    processFrame(it)
                    it.close()
                }

                // Capture at ~10 FPS
                delay(100)
            } catch (e: Exception) {
                Log.e(TAG, "Error capturing frame", e)
            }
        }
    }

    private fun processFrame(image: Image) {
        val planes = image.planes
        val buffer = planes[0].buffer
        val pixelStride = planes[0].pixelStride
        val rowStride = planes[0].rowStride
        val rowPadding = rowStride - pixelStride * screenWidth

        // Create bitmap from image
        val bitmap = Bitmap.createBitmap(
            screenWidth + rowPadding / pixelStride,
            screenHeight,
            Bitmap.Config.ARGB_8888
        )
        bitmap.copyPixelsFromBuffer(buffer)

        // Crop to actual screen size if needed
        val croppedBitmap = if (rowPadding > 0) {
            Bitmap.createBitmap(bitmap, 0, 0, screenWidth, screenHeight)
        } else {
            bitmap
        }

        // Convert to byte array for Rust
        val byteBuffer = ByteBuffer.allocate(croppedBitmap.byteCount)
        croppedBitmap.copyPixelsToBuffer(byteBuffer)
        val frameData = byteBuffer.array()

        // Process frame through Sheba core
        val actionCode = ShebaCore.processFrame(frameData, screenWidth, screenHeight)

        // Execute action if any
        if (actionCode != 0L) {
            val action = ShebaCore.decodeAction(actionCode)
            ShebaAccessibilityService.instance?.executeAction(action)
        }

        // Clean up
        if (croppedBitmap != bitmap) {
            bitmap.recycle()
        }
        croppedBitmap.recycle()
    }
}
