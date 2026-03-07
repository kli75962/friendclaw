package com.uty.phoneclaw

import android.Manifest
import android.accessibilityservice.AccessibilityServiceInfo
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Bundle
import android.provider.Settings
import android.view.accessibility.AccessibilityManager
import androidx.activity.enableEdgeToEdge
import androidx.appcompat.app.AlertDialog
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {

  companion object {
    private const val REQUEST_CAMERA = 1001
    private const val PREFS_NAME = "phoneclaw_prefs"
    private const val KEY_PERMISSIONS_ASKED = "permissions_asked"
  }

  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)

    ViewCompat.setOnApplyWindowInsetsListener(findViewById(android.R.id.content)) { view, insets ->
      val systemBars = insets.getInsets(WindowInsetsCompat.Type.systemBars())
      view.setPadding(systemBars.left, systemBars.top, systemBars.right, systemBars.bottom)
      insets
    }

    val prefs = getSharedPreferences(PREFS_NAME, MODE_PRIVATE)
    if (!prefs.getBoolean(KEY_PERMISSIONS_ASKED, false)) {
      prefs.edit().putBoolean(KEY_PERMISSIONS_ASKED, true).apply()
      requestInitialPermissions()
    }
  }

  override fun onResume() {
    super.onResume()
    // If accessibility is still not enabled after returning from Settings, show dialog again
    if (!isAccessibilityEnabled()) {
      showAccessibilityDialog()
    }
  }

  private fun requestInitialPermissions() {
    if (ContextCompat.checkSelfPermission(this, Manifest.permission.CAMERA)
      != PackageManager.PERMISSION_GRANTED) {
      ActivityCompat.requestPermissions(this, arrayOf(Manifest.permission.CAMERA), REQUEST_CAMERA)
    } else {
      checkAccessibilityOnce()
    }
  }

  override fun onRequestPermissionsResult(
    requestCode: Int, permissions: Array<out String>, grantResults: IntArray
  ) {
    super.onRequestPermissionsResult(requestCode, permissions, grantResults)
    if (requestCode == REQUEST_CAMERA) {
      checkAccessibilityOnce()
    }
  }

  private fun checkAccessibilityOnce() {
    if (!isAccessibilityEnabled()) {
      showAccessibilityDialog()
    }
  }

  private fun isAccessibilityEnabled(): Boolean {
    val am = getSystemService(ACCESSIBILITY_SERVICE) as AccessibilityManager
    val enabled = am.getEnabledAccessibilityServiceList(AccessibilityServiceInfo.FEEDBACK_ALL_MASK)
    return enabled.any { it.resolveInfo.serviceInfo.packageName == packageName }
  }

  private fun showAccessibilityDialog() {
    AlertDialog.Builder(this)
      .setTitle("Accessibility access needed")
      .setMessage("PhoneClaw needs the Accessibility Service to control your phone on your behalf. Please enable it in Settings → Accessibility → PhoneClaw.")
      .setPositiveButton("Open Settings") { _, _ ->
        startActivity(Intent(Settings.ACTION_ACCESSIBILITY_SETTINGS))
      }
      .setNegativeButton("Later", null)
      .setCancelable(true)
      .show()
  }
}
