// SPDX-License-Identifier: MIT
// Mechanical SDL platform adapter. Product and native lifetime policy lives in Rust.
package com.machinerhythm.xbx;

public final class XbxActivity extends org.libsdl.app.SDLActivity {
    private static native void nativeSetForeground(boolean foreground);

    @Override protected String[] getLibraries() {
        return new String[] { "SDL3", "xemu", "xbx" };
    }
    @Override protected void onPause() {
        if (!mBrokenLibraries) nativeSetForeground(false);
        super.onPause();
    }
    @Override protected void onResume() {
        super.onResume();
        if (!mBrokenLibraries) nativeSetForeground(true);
    }
}
