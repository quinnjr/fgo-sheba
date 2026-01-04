package io.sheba.stealth

import android.app.ActivityManager
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.util.Log

/**
 * Receives broadcasts for stealth mode changes.
 */
class StealthReceiver : BroadcastReceiver() {

    companion object {
        private const val TAG = "StealthReceiver"
    }

    override fun onReceive(context: Context?, intent: Intent?) {
        if (intent?.action == "io.sheba.UPDATE_RECENTS_VISIBILITY") {
            val exclude = intent.getBooleanExtra("exclude", false)
            Log.d(TAG, "Received recents visibility update: exclude=$exclude")

            // Store preference - MainActivity will check this on resume
            context?.getSharedPreferences("stealth_prefs", Context.MODE_PRIVATE)
                ?.edit()
                ?.putBoolean("exclude_from_recents", exclude)
                ?.apply()
        }
    }
}
