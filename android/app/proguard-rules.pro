# Sheba ProGuard Rules

# Keep JNI methods
-keepclasseswithmembernames class * {
    native <methods>;
}

# Keep ShebaCore class
-keep class io.sheba.ShebaCore { *; }
-keep class io.sheba.ShebaAction { *; }
-keep class io.sheba.ShebaAction$* { *; }

# Keep Kotlin coroutines
-keepnames class kotlinx.coroutines.** { *; }
-dontwarn kotlinx.coroutines.**

# Keep accessibility service
-keep class io.sheba.ShebaAccessibilityService { *; }
