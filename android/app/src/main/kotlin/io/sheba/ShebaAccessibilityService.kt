package io.sheba

import android.accessibilityservice.AccessibilityService
import android.accessibilityservice.GestureDescription
import android.graphics.Path
import android.graphics.Rect
import android.util.Log
import android.view.accessibility.AccessibilityEvent
import android.view.accessibility.AccessibilityNodeInfo
import kotlinx.coroutines.*

/**
 * Accessibility Service for FGO Sheba automation.
 *
 * This service provides:
 * - Touch gesture injection
 * - Screen content access
 * - Window state monitoring
 */
class ShebaAccessibilityService : AccessibilityService() {

    companion object {
        private const val TAG = "ShebaAccessibility"

        // Singleton instance for access from other components
        @Volatile
        var instance: ShebaAccessibilityService? = null
            private set
    }

    private val serviceScope = CoroutineScope(Dispatchers.Default + SupervisorJob())

    override fun onCreate() {
        super.onCreate()
        instance = this
        Log.i(TAG, "Sheba Accessibility Service created")
    }

    override fun onDestroy() {
        super.onDestroy()
        instance = null
        serviceScope.cancel()
        Log.i(TAG, "Sheba Accessibility Service destroyed")
    }

    override fun onServiceConnected() {
        super.onServiceConnected()
        Log.i(TAG, "Sheba Accessibility Service connected")
    }

    override fun onAccessibilityEvent(event: AccessibilityEvent?) {
        // Monitor for FGO window changes
        event?.let {
            if (it.packageName?.toString()?.contains("fategrandorder") == true ||
                it.packageName?.toString()?.contains("aniplex") == true) {
                when (it.eventType) {
                    AccessibilityEvent.TYPE_WINDOW_STATE_CHANGED -> {
                        Log.d(TAG, "FGO window state changed")
                    }
                    AccessibilityEvent.TYPE_WINDOW_CONTENT_CHANGED -> {
                        // Content changed
                    }
                }
            }
        }
    }

    override fun onInterrupt() {
        Log.w(TAG, "Sheba Accessibility Service interrupted")
    }

    /**
     * Execute a Sheba action.
     */
    fun executeAction(action: ShebaAction) {
        serviceScope.launch {
            when (action) {
                is ShebaAction.Tap -> performTap(action.x.toFloat(), action.y.toFloat())
                is ShebaAction.Swipe -> performSwipe(
                    action.startX.toFloat(),
                    action.startY.toFloat(),
                    action.endX.toFloat(),
                    action.endY.toFloat(),
                    action.durationMs
                )
                is ShebaAction.Wait -> delay(action.durationMs)
                is ShebaAction.SelectCards -> selectCards(action.cardIndices)
                is ShebaAction.UseSkill -> useSkill(action.servantIdx, action.skillIdx, action.target)
                is ShebaAction.UseNP -> useNP(action.servantIdx)
                is ShebaAction.TargetEnemy -> targetEnemy(action.enemyIdx)
                is ShebaAction.TapAttack -> tapAttack()
                is ShebaAction.UseMasterSkill -> useMasterSkill(action.skillIdx, action.target)
                ShebaAction.None -> { /* No action */ }
            }
        }
    }

    /**
     * Perform a tap gesture at the specified coordinates.
     */
    suspend fun performTap(x: Float, y: Float): Boolean = suspendCancellableCoroutine { cont ->
        val path = Path().apply {
            moveTo(x, y)
        }

        val gesture = GestureDescription.Builder()
            .addStroke(GestureDescription.StrokeDescription(path, 0, 50))
            .build()

        val callback = object : GestureResultCallback() {
            override fun onCompleted(gestureDescription: GestureDescription?) {
                Log.d(TAG, "Tap completed at ($x, $y)")
                cont.resumeWith(Result.success(true))
            }

            override fun onCancelled(gestureDescription: GestureDescription?) {
                Log.w(TAG, "Tap cancelled at ($x, $y)")
                cont.resumeWith(Result.success(false))
            }
        }

        dispatchGesture(gesture, callback, null)
    }

    /**
     * Perform a swipe gesture.
     */
    suspend fun performSwipe(
        startX: Float,
        startY: Float,
        endX: Float,
        endY: Float,
        durationMs: Long
    ): Boolean = suspendCancellableCoroutine { cont ->
        val path = Path().apply {
            moveTo(startX, startY)
            lineTo(endX, endY)
        }

        val gesture = GestureDescription.Builder()
            .addStroke(GestureDescription.StrokeDescription(path, 0, durationMs))
            .build()

        val callback = object : GestureResultCallback() {
            override fun onCompleted(gestureDescription: GestureDescription?) {
                Log.d(TAG, "Swipe completed")
                cont.resumeWith(Result.success(true))
            }

            override fun onCancelled(gestureDescription: GestureDescription?) {
                Log.w(TAG, "Swipe cancelled")
                cont.resumeWith(Result.success(false))
            }
        }

        dispatchGesture(gesture, callback, null)
    }

    /**
     * Select cards in sequence.
     */
    private suspend fun selectCards(cardIndices: List<Int>) {
        for (idx in cardIndices) {
            val coords = getCardCoordinates(idx)
            performTap(coords.first, coords.second)
            delay(200)
        }
    }

    /**
     * Use a servant skill.
     */
    private suspend fun useSkill(servantIdx: Int, skillIdx: Int, target: Int?) {
        val coords = getSkillCoordinates(servantIdx, skillIdx)
        performTap(coords.first, coords.second)
        delay(300)

        target?.let {
            val targetCoords = getServantTargetCoordinates(it)
            performTap(targetCoords.first, targetCoords.second)
            delay(200)
        }
    }

    /**
     * Use a Noble Phantasm.
     */
    private suspend fun useNP(servantIdx: Int) {
        val coords = getNPCoordinates(servantIdx)
        performTap(coords.first, coords.second)
        delay(200)
    }

    /**
     * Target an enemy.
     */
    private suspend fun targetEnemy(enemyIdx: Int) {
        val coords = getEnemyCoordinates(enemyIdx)
        performTap(coords.first, coords.second)
        delay(200)
    }

    /**
     * Tap the attack button.
     */
    private suspend fun tapAttack() {
        val coords = getAttackButtonCoordinates()
        performTap(coords.first, coords.second)
        delay(200)
    }

    /**
     * Use a master skill.
     */
    private suspend fun useMasterSkill(skillIdx: Int, target: Int?) {
        // Open master skill menu
        performTap(1880f * screenScale(), 440f * screenScale())
        delay(500)

        // Tap specific skill
        val coords = getMasterSkillCoordinates(skillIdx)
        performTap(coords.first, coords.second)
        delay(300)

        target?.let {
            val targetCoords = getServantTargetCoordinates(it)
            performTap(targetCoords.first, targetCoords.second)
            delay(200)
        }
    }

    // Screen coordinate helpers (scaled from 1920x1080 reference)
    private fun screenScale(): Float {
        val metrics = resources.displayMetrics
        return metrics.widthPixels / 1920f
    }

    private fun getCardCoordinates(idx: Int): Pair<Float, Float> {
        val scale = screenScale()
        val baseX = 180 + idx * 300 + 150 // Center of card
        val y = 880
        return Pair(baseX * scale, y * scale)
    }

    private fun getNPCoordinates(servantIdx: Int): Pair<Float, Float> {
        val scale = screenScale()
        val baseX = 380 + servantIdx * 280 + 140
        val y = 320
        return Pair(baseX * scale, y * scale)
    }

    private fun getSkillCoordinates(servantIdx: Int, skillIdx: Int): Pair<Float, Float> {
        val scale = screenScale()
        val servantX = 160 + servantIdx * 350
        val skillX = servantX + skillIdx * 85
        val y = 950
        return Pair(skillX * scale, y * scale)
    }

    private fun getEnemyCoordinates(enemyIdx: Int): Pair<Float, Float> {
        val scale = screenScale()
        val x = 400 + enemyIdx * 350 + 175
        val y = 150
        return Pair(x * scale, y * scale)
    }

    private fun getAttackButtonCoordinates(): Pair<Float, Float> {
        val scale = screenScale()
        return Pair(1700f * scale, 500f * scale)
    }

    private fun getMasterSkillCoordinates(skillIdx: Int): Pair<Float, Float> {
        val scale = screenScale()
        return Pair(1820f * scale, (340 + skillIdx * 80) * scale)
    }

    private fun getServantTargetCoordinates(servantIdx: Int): Pair<Float, Float> {
        val scale = screenScale()
        val x = 400 + servantIdx * 380 + 190
        val y = 540
        return Pair(x * scale, y * scale)
    }
}
