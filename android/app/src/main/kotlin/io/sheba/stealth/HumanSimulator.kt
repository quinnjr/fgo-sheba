package io.sheba.stealth

import kotlin.math.pow
import kotlin.math.sqrt
import kotlin.random.Random

/**
 * Simulates human-like behavior patterns to avoid bot detection.
 *
 * Games use various heuristics to detect automation:
 * - Perfectly consistent timing between actions
 * - Exact same tap coordinates every time
 * - Impossibly fast reactions
 * - Actions during impossible times (instant menu navigation)
 * - Linear/predictable movement patterns
 *
 * This class adds realistic variance to all automated actions.
 */
object HumanSimulator {

    // Typical human reaction time range (ms)
    private const val MIN_REACTION_TIME = 180L
    private const val MAX_REACTION_TIME = 350L

    // Time to visually process screen changes (ms)
    private const val VISUAL_PROCESSING_MIN = 100L
    private const val VISUAL_PROCESSING_MAX = 300L

    // Finger movement time estimates (ms)
    private const val TAP_DURATION_MIN = 50L
    private const val TAP_DURATION_MAX = 150L

    // Swipe characteristics
    private const val SWIPE_DURATION_MIN = 150L
    private const val SWIPE_DURATION_MAX = 400L

    /**
     * Generate a human-like delay before performing an action.
     * Uses a combination of reaction time + processing time.
     */
    fun getActionDelay(): Long {
        val reactionTime = Random.nextLong(MIN_REACTION_TIME, MAX_REACTION_TIME)
        val processingTime = Random.nextLong(VISUAL_PROCESSING_MIN, VISUAL_PROCESSING_MAX)

        // Occasionally add "hesitation" (5% chance)
        val hesitation = if (Random.nextFloat() < 0.05f) {
            Random.nextLong(200, 800)
        } else {
            0L
        }

        return reactionTime + processingTime + hesitation
    }

    /**
     * Generate delay between consecutive actions (like selecting multiple cards)
     */
    fun getConsecutiveActionDelay(): Long {
        // Faster than initial action but still human-like
        return Random.nextLong(80, 250)
    }

    /**
     * Generate tap duration (how long finger stays on screen)
     */
    fun getTapDuration(): Long {
        return Random.nextLong(TAP_DURATION_MIN, TAP_DURATION_MAX)
    }

    /**
     * Generate realistic tap position with slight offset from target.
     * Humans don't tap exactly on the center every time.
     */
    fun humanizeTapPosition(targetX: Int, targetY: Int, accuracy: Float = 0.9f): Pair<Int, Int> {
        // Higher accuracy = smaller offset
        val maxOffset = ((1f - accuracy) * 30).toInt().coerceAtLeast(2)

        // Use gaussian distribution for more realistic spread
        val offsetX = (Random.nextGaussian() * maxOffset / 2).toInt()
        val offsetY = (Random.nextGaussian() * maxOffset / 2).toInt()

        return Pair(targetX + offsetX, targetY + offsetY)
    }

    /**
     * Generate gaussian random number (Box-Muller transform)
     */
    private fun Random.nextGaussian(): Double {
        var v1: Double
        var v2: Double
        var s: Double
        do {
            v1 = 2 * nextDouble() - 1
            v2 = 2 * nextDouble() - 1
            s = v1 * v1 + v2 * v2
        } while (s >= 1 || s == 0.0)

        val multiplier = sqrt(-2 * kotlin.math.ln(s) / s)
        return v1 * multiplier
    }

    /**
     * Generate swipe parameters with human-like characteristics
     */
    fun humanizeSwipe(
        startX: Int, startY: Int,
        endX: Int, endY: Int
    ): SwipeParams {
        // Add slight randomness to start/end points
        val (humanStartX, humanStartY) = humanizeTapPosition(startX, startY, 0.95f)
        val (humanEndX, humanEndY) = humanizeTapPosition(endX, endY, 0.85f)

        // Calculate distance-appropriate duration
        val distance = sqrt(
            (endX - startX).toDouble().pow(2) +
            (endY - startY).toDouble().pow(2)
        )

        // Longer swipes take more time
        val baseDuration = SWIPE_DURATION_MIN + (distance * 0.3).toLong()
        val duration = Random.nextLong(
            baseDuration,
            (baseDuration * 1.3).toLong().coerceAtMost(SWIPE_DURATION_MAX)
        )

        // Generate intermediate points for curved path (humans don't swipe in straight lines)
        val curveOffset = Random.nextInt(-20, 21)
        val midX = (humanStartX + humanEndX) / 2 + curveOffset
        val midY = (humanStartY + humanEndY) / 2 + curveOffset

        return SwipeParams(
            startX = humanStartX,
            startY = humanStartY,
            endX = humanEndX,
            endY = humanEndY,
            midX = midX,
            midY = midY,
            duration = duration
        )
    }

    /**
     * Determine if we should add a "micro-pause" (simulates human attention drift)
     */
    fun shouldAddMicroPause(): Boolean {
        return Random.nextFloat() < 0.08f // 8% chance
    }

    /**
     * Get micro-pause duration
     */
    fun getMicroPauseDuration(): Long {
        return Random.nextLong(500, 2000)
    }

    /**
     * Simulate the time a human takes to recognize card types and make selection
     */
    fun getCardSelectionThinkTime(): Long {
        // First card selection takes longer (evaluating options)
        return Random.nextLong(300, 800)
    }

    /**
     * Simulate time to recognize and react to NP gauge being full
     */
    fun getNPRecognitionDelay(): Long {
        return Random.nextLong(200, 500)
    }

    /**
     * Simulate delay before confirming an important action
     */
    fun getConfirmationDelay(): Long {
        // Humans pause briefly before important buttons
        return Random.nextLong(150, 400)
    }

    /**
     * Add periodic longer breaks to simulate real player behavior
     * Returns true if a break should be taken
     */
    fun shouldTakeBreak(battlesCompleted: Int): Boolean {
        // After every 5-10 battles, small chance of break
        if (battlesCompleted > 0 && battlesCompleted % 5 == 0) {
            return Random.nextFloat() < 0.15f
        }
        return false
    }

    /**
     * Get break duration (simulating player checking phone, etc.)
     */
    fun getBreakDuration(): Long {
        return Random.nextLong(3000, 10000)
    }
}

/**
 * Parameters for a humanized swipe gesture
 */
data class SwipeParams(
    val startX: Int,
    val startY: Int,
    val endX: Int,
    val endY: Int,
    val midX: Int,  // Intermediate point for curve
    val midY: Int,
    val duration: Long
)
