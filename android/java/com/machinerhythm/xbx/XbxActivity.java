// SPDX-License-Identifier: MIT
// Mechanical document, Canvas, audio-focus and SDL adapter. Player policy is Rust.
package com.machinerhythm.xbx;

import android.app.Activity;
import android.content.Intent;
import android.graphics.Canvas;
import android.graphics.Color;
import android.graphics.Paint;
import android.media.AudioAttributes;
import android.media.AudioFocusRequest;
import android.media.AudioManager;
import android.net.Uri;
import android.os.Bundle;
import android.os.ParcelFileDescriptor;
import android.view.Gravity;
import android.view.MotionEvent;
import android.view.View;
import android.view.ViewGroup;
import android.widget.RelativeLayout;
import android.widget.TextView;

public final class XbxActivity extends org.libsdl.app.SDLActivity {
    private static native void nativeSetForeground(boolean foreground);
    private static native void nativeSetAudioFocus(boolean focused);
    private static native void nativeDocument(int fd, String uri, String error);
    private static native void nativeTouch(int id, int phase, float x, float y, float w, float h, float density);
    private static native float[] nativeLayout(int index, float w, float h, float density);
    private static native String nativeLabel(int index);
    private static native void nativeNeutral();
    private Controls controls;
    private TextView status;
    private AudioManager audio;
    private AudioFocusRequest focus;

    @Override protected String[] getLibraries() { return new String[] { "SDL3", "xemu", "xbx" }; }

    @Override protected void onCreate(Bundle state) {
        super.onCreate(state);
        if (mBrokenLibraries || mLayout == null) return;
        controls = new Controls();
        mLayout.addView(controls, new ViewGroup.LayoutParams(-1, -1));
        status = new TextView(this);
        status.setTextColor(Color.WHITE);
        status.setBackgroundColor(0xcc000000);
        status.setGravity(Gravity.CENTER);
        status.setText("");
        mLayout.addView(status, new ViewGroup.LayoutParams(-1, -2));
        audio = (AudioManager)getSystemService(AUDIO_SERVICE);
        focus = new AudioFocusRequest.Builder(AudioManager.AUDIOFOCUS_GAIN)
            .setAudioAttributes(new AudioAttributes.Builder().setUsage(AudioAttributes.USAGE_GAME)
                .setContentType(AudioAttributes.CONTENT_TYPE_MUSIC).build())
            .setOnAudioFocusChangeListener(new AudioManager.OnAudioFocusChangeListener() {
                @Override public void onAudioFocusChange(int change) { nativeSetAudioFocus(change == AudioManager.AUDIOFOCUS_GAIN); }
            })
            .build();
    }

    public void showStatus(String text) {
        runOnUiThread(new Runnable() { @Override public void run() {
            if (status != null) { status.setText(text); status.setVisibility(text.isEmpty() ? View.GONE : View.VISIBLE); }
            if (controls != null) { controls.surfaceRect(); controls.invalidate(); }
        }});
    }

    public void openDocument(String saved) {
        runOnUiThread(new Runnable() { @Override public void run() {
            if (saved != null && !saved.isEmpty()) {
                try { deliver(Uri.parse(saved)); return; }
                catch (Exception ignored) { nativeDocument(-1, null, "grant-unavailable"); return; }
            }
            Intent intent = new Intent(Intent.ACTION_OPEN_DOCUMENT);
            intent.addCategory(Intent.CATEGORY_OPENABLE);
            intent.setType("*/*");
            intent.addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION | Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION);
            startActivityForResult(intent, 701);
        }});
    }

    private void deliver(Uri uri) throws Exception {
        try (ParcelFileDescriptor fd = getContentResolver().openFileDescriptor(uri, "r")) {
            if (fd == null) throw new IllegalStateException("Provider returned no readable object");
            nativeDocument(fd.detachFd(), uri.toString(), null);
        }
    }

    @Override protected void onActivityResult(int request, int result, Intent data) {
        if (request != 701) { super.onActivityResult(request, result, data); return; }
        if (result != Activity.RESULT_OK || data == null || data.getData() == null) {
            nativeDocument(-1, null, "Game selection canceled"); return;
        }
        try {
            Uri uri = data.getData();
            getContentResolver().takePersistableUriPermission(uri, Intent.FLAG_GRANT_READ_URI_PERMISSION);
            deliver(uri);
        } catch (Exception error) {
            nativeDocument(-1, null, "The provider must retain a readable local document grant (" + error.getClass().getSimpleName() + ")");
        }
    }

    @Override protected void onPause() {
        if (!mBrokenLibraries) { nativeSetForeground(false); nativeNeutral(); }
        if (audio != null && focus != null) audio.abandonAudioFocusRequest(focus);
        super.onPause();
    }
    @Override protected void onResume() {
        super.onResume();
        if (!mBrokenLibraries) {
            nativeSetAudioFocus(audio != null && focus != null && audio.requestAudioFocus(focus) == AudioManager.AUDIOFOCUS_REQUEST_GRANTED);
            nativeSetForeground(true);
        }
    }

    private final class Controls extends View {
        private final Paint paint = new Paint(Paint.ANTI_ALIAS_FLAG);
        Controls() { super(XbxActivity.this); setClickable(true); }
        private float density() { return getResources().getDisplayMetrics().density; }
        void surfaceRect() {
            float[] r = nativeLayout(-1, getWidth(), getHeight(), density());
            if (r[2] < 1 || r[3] < 1) return;
            RelativeLayout.LayoutParams p = new RelativeLayout.LayoutParams((int)r[2], (int)r[3]);
            p.leftMargin = (int)r[0]; p.topMargin = (int)r[1];
            ViewGroup.LayoutParams old = mSurface.getLayoutParams();
            if (old instanceof RelativeLayout.LayoutParams) {
                RelativeLayout.LayoutParams q = (RelativeLayout.LayoutParams)old;
                if (q.width == p.width && q.height == p.height && q.leftMargin == p.leftMargin && q.topMargin == p.topMargin) return;
            }
            mSurface.setLayoutParams(p);
        }
        @Override protected void onSizeChanged(int w, int h, int oldw, int oldh) {
            nativeNeutral(); surfaceRect();
        }
        @Override protected void onDraw(Canvas canvas) {
            int count = (int)nativeLayout(-2, getWidth(), getHeight(), density())[0];
            for (int i=0; i<count; i++) {
                float[] p = nativeLayout(i, getWidth(), getHeight(), density());
                paint.setColor(p[3] != 0 ? 0xcc6ca9dc : 0xaa303945);
                canvas.drawCircle(p[0], p[1], p[2], paint);
                paint.setColor(Color.WHITE);
                paint.setTextAlign(Paint.Align.CENTER);
                paint.setTextSize(p[2] * 0.68f);
                canvas.drawText(nativeLabel(i), p[0], p[1] - (paint.ascent()+paint.descent())/2, paint);
            }
        }
        @Override public boolean onTouchEvent(MotionEvent event) {
            int action = event.getActionMasked();
            if (action == MotionEvent.ACTION_CANCEL) { nativeNeutral(); invalidate(); return true; }
            for (int i=0; i<event.getPointerCount(); i++) {
                int phase = 1;
                if (i == event.getActionIndex()) {
                    if (action == MotionEvent.ACTION_DOWN || action == MotionEvent.ACTION_POINTER_DOWN) phase = 0;
                    if (action == MotionEvent.ACTION_UP || action == MotionEvent.ACTION_POINTER_UP) phase = 2;
                }
                nativeTouch(event.getPointerId(i), phase, event.getX(i), event.getY(i), getWidth(), getHeight(), density());
            }
            invalidate(); return true;
        }
    }
}
