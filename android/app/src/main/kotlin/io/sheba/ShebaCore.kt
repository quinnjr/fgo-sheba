package io.sheba

/**
 * JNI bridge to the Rust automation core.
 *
 * This class provides the interface between the Android accessibility service
 * and the Rust-based automation engine.
 */
object ShebaCore {

    init {
        System.loadLibrary("fgo_sheba")
    }

    /**
     * Initialize the Sheba automation engine.
     * @param configJson JSON string containing settings
     * @return true if initialization was successful
     */
    external fun init(configJson: String?): Boolean

    /**
     * Process a screen frame and get the next action.
     * @param frameData Raw RGBA pixel data
     * @param width Frame width
     * @param height Frame height
     * @return Encoded action value
     */
    external fun processFrame(frameData: ByteArray, width: Int, height: Int): Long

    /**
     * Get the current game state as JSON.
     * @return JSON string representing game state
     */
    external fun getGameState(): String

    /**
     * Update settings.
     * @param settingsJson JSON string containing new settings
     * @return true if update was successful
     */
    external fun updateSettings(settingsJson: String): Boolean

    /**
     * Pause or resume automation.
     * @param paused true to pause, false to resume
     */
    external fun setPaused(paused: Boolean)

    /**
     * Get the action type from an encoded action.
     * @param actionCode Encoded action value
     * @return Action type code
     */
    external fun getActionType(actionCode: Long): Int

    /**
     * Get action X coordinate.
     * @param actionCode Encoded action value
     * @return X coordinate
     */
    external fun getActionX(actionCode: Long): Int

    /**
     * Get action Y coordinate.
     * @param actionCode Encoded action value
     * @return Y coordinate
     */
    external fun getActionY(actionCode: Long): Int

    /**
     * Get action data (duration, index, etc.).
     * @param actionCode Encoded action value
     * @return Extra data value
     */
    external fun getActionData(actionCode: Long): Int

    // Action type constants
    object ActionType {
        const val NONE = 0
        const val TAP = 1
        const val SWIPE = 2
        const val WAIT = 3
        const val SELECT_CARDS = 4
        const val USE_SKILL = 5
        const val USE_NP = 6
        const val TARGET_ENEMY = 7
        const val TAP_ATTACK = 8
        const val USE_MASTER_SKILL = 9
    }

    /**
     * Decode an action from the encoded value.
     */
    fun decodeAction(actionCode: Long): ShebaAction {
        val type = getActionType(actionCode)

        return when (type) {
            ActionType.NONE -> ShebaAction.None
            ActionType.TAP -> ShebaAction.Tap(getActionX(actionCode), getActionY(actionCode))
            ActionType.SWIPE -> ShebaAction.Swipe(
                getActionX(actionCode),
                getActionY(actionCode),
                0, 0, // End coords need separate handling
                getActionData(actionCode).toLong()
            )
            ActionType.WAIT -> ShebaAction.Wait(actionCode and 0xFFFFFFFF)
            ActionType.SELECT_CARDS -> {
                val c0 = ((actionCode shr 16) and 0xFF).toInt()
                val c1 = ((actionCode shr 8) and 0xFF).toInt()
                val c2 = (actionCode and 0xFF).toInt()
                ShebaAction.SelectCards(listOf(c0, c1, c2))
            }
            ActionType.USE_SKILL -> {
                val servant = ((actionCode shr 16) and 0xFF).toInt()
                val skill = ((actionCode shr 8) and 0xFF).toInt()
                val target = (actionCode and 0xFF).toInt().takeIf { it != 255 }
                ShebaAction.UseSkill(servant, skill, target)
            }
            ActionType.USE_NP -> ShebaAction.UseNP((actionCode and 0xFF).toInt())
            ActionType.TARGET_ENEMY -> ShebaAction.TargetEnemy((actionCode and 0xFF).toInt())
            ActionType.TAP_ATTACK -> ShebaAction.TapAttack
            ActionType.USE_MASTER_SKILL -> {
                val skill = ((actionCode shr 8) and 0xFF).toInt()
                val target = (actionCode and 0xFF).toInt().takeIf { it != 255 }
                ShebaAction.UseMasterSkill(skill, target)
            }
            else -> ShebaAction.None
        }
    }
}

/**
 * Sealed class representing actions the automation can take.
 */
sealed class ShebaAction {
    object None : ShebaAction()
    data class Tap(val x: Int, val y: Int) : ShebaAction()
    data class Swipe(val startX: Int, val startY: Int, val endX: Int, val endY: Int, val durationMs: Long) : ShebaAction()
    data class Wait(val durationMs: Long) : ShebaAction()
    data class SelectCards(val cardIndices: List<Int>) : ShebaAction()
    data class UseSkill(val servantIdx: Int, val skillIdx: Int, val target: Int?) : ShebaAction()
    data class UseNP(val servantIdx: Int) : ShebaAction()
    data class TargetEnemy(val enemyIdx: Int) : ShebaAction()
    object TapAttack : ShebaAction()
    data class UseMasterSkill(val skillIdx: Int, val target: Int?) : ShebaAction()
}
