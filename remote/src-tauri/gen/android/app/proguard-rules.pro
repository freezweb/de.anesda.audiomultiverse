# Add project specific ProGuard rules here.

# Keep Tauri classes
-keep class app.tauri.** { *; }
-keep class de.anesda.audiomultiverse.remote.** { *; }

# WebView
-keepclassmembers class * extends android.webkit.WebViewClient {
    public void *(android.webkit.WebView, java.lang.String, android.graphics.Bitmap);
    public boolean *(android.webkit.WebView, java.lang.String);
}
-keepclassmembers class * extends android.webkit.WebViewClient {
    public void *(android.webkit.WebView, java.lang.String);
}

# Keep native methods
-keepclasseswithmembernames class * {
    native <methods>;
}
