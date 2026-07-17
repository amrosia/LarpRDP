// SPDX-License-Identifier: AGPL-3.0

#include "rdp_bridge.h"

#include <freerdp/freerdp.h>
#include <freerdp/client.h>
#include <freerdp/input.h>
#include <freerdp/event.h>
#include <freerdp/settings.h>
#include <freerdp/scancode.h>
#include <freerdp/gdi/gdi.h>
#include <freerdp/gdi/dc.h>
#include <freerdp/codec/color.h>

#include <winpr/wtypes.h>
#include <winpr/synch.h>
#include <winpr/thread.h>

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <time.h>

struct RdpBridge {
    freerdp* instance;
    rdpContext* context;
    rdpSettings* settings;
    int connected;
    int active;
    char last_error[512];
};

/* --- Callbacks --- */

static BOOL bridge_authenticate_ex(freerdp* instance,
                                   char** username, char** password, char** domain,
                                   rdp_auth_reason reason)
{
    /* Credentials already set via settings; just confirm they are valid */
    (void)instance;
    (void)reason;
    if (*username && *password)
        return TRUE;
    return FALSE;
}

static DWORD bridge_verify_certificate_ex(freerdp* instance,
                                          const char* host, UINT16 port,
                                          const char* common_name, const char* subject,
                                          const char* issuer, const char* fingerprint,
                                          DWORD flags)
{
    /* Accept all certificates for PoC (equivalent to /cert:ignore) */
    (void)instance;
    (void)host;
    (void)port;
    (void)common_name;
    (void)subject;
    (void)issuer;
    (void)fingerprint;
    (void)flags;
    return 2; /* accept for this session */
}

static DWORD bridge_verify_changed_certificate_ex(freerdp* instance,
                                                  const char* host, UINT16 port,
                                                  const char* common_name, const char* subject,
                                                  const char* issuer, const char* new_fingerprint,
                                                  const char* old_subject, const char* old_issuer,
                                                  const char* old_fingerprint, DWORD flags)
{
    (void)instance;
    (void)host;
    (void)port;
    (void)common_name;
    (void)subject;
    (void)issuer;
    (void)new_fingerprint;
    (void)old_subject;
    (void)old_issuer;
    (void)old_fingerprint;
    (void)flags;
    return 2; /* accept for this session */
}

/* --- Public API --- */

RdpBridge* rdp_bridge_create(void)
{
    RdpBridge* bridge = calloc(1, sizeof(RdpBridge));
    if (!bridge)
        return NULL;

    bridge->instance = freerdp_new();
    if (!bridge->instance) {
        free(bridge);
        return NULL;
    }

    /* Register callbacks */
    bridge->instance->AuthenticateEx = bridge_authenticate_ex;
    bridge->instance->VerifyCertificateEx = bridge_verify_certificate_ex;
    bridge->instance->VerifyChangedCertificateEx = bridge_verify_changed_certificate_ex;

    /* Allocate context (must be done before accessing settings) */
    if (!freerdp_context_new(bridge->instance)) {
        snprintf(bridge->last_error, sizeof(bridge->last_error),
                 "freerdp_context_new failed");
        freerdp_free(bridge->instance);
        free(bridge);
        return NULL;
    }

    bridge->context = bridge->instance->context;
    bridge->settings = bridge->context->settings;

    /* Disable unwanted features */
    freerdp_settings_set_bool(bridge->settings, FreeRDP_AutoAcceptCertificate, TRUE);
    freerdp_settings_set_bool(bridge->settings, FreeRDP_IgnoreCertificate, TRUE);

    /* Disable bitmap cache for minimal overhead */
    freerdp_settings_set_bool(bridge->settings, FreeRDP_BitmapCacheEnabled, FALSE);

    /* Disable unwanted features */
    freerdp_settings_set_bool(bridge->settings, FreeRDP_AudioPlayback, FALSE);
    freerdp_settings_set_bool(bridge->settings, FreeRDP_AudioCapture, FALSE);
    freerdp_settings_set_bool(bridge->settings, FreeRDP_DeviceRedirection, FALSE);
    freerdp_settings_set_bool(bridge->settings, FreeRDP_RedirectClipboard, FALSE);

    return bridge;
}

int rdp_bridge_set_server(RdpBridge* bridge, const char* server)
{
    if (!bridge || !server)
        return -1;

    /* Parse optional port from server (server:port format) */
    char* colon = strchr(server, ':');
    if (colon) {
        size_t hostlen = colon - server;
        char* host = strndup(server, hostlen);
        int port = atoi(colon + 1);

        freerdp_settings_set_string(bridge->settings, FreeRDP_ServerHostname, host);
        freerdp_settings_set_uint32(bridge->settings, FreeRDP_ServerPort, (UINT32)port);
        free(host);
    } else {
        freerdp_settings_set_string(bridge->settings, FreeRDP_ServerHostname, server);
        freerdp_settings_set_uint32(bridge->settings, FreeRDP_ServerPort, 3389);
    }

    return 0;
}

int rdp_bridge_set_credentials(RdpBridge* bridge,
                               const char* username,
                               const char* password,
                               const char* domain)
{
    if (!bridge)
        return -1;

    if (username)
        freerdp_settings_set_string(bridge->settings, FreeRDP_Username, username);

    if (password)
        freerdp_settings_set_string(bridge->settings, FreeRDP_Password, password);

    if (domain)
        freerdp_settings_set_string(bridge->settings, FreeRDP_Domain, domain);
    else
        freerdp_settings_set_string(bridge->settings, FreeRDP_Domain, ".");

    return 0;
}

int rdp_bridge_enable_pth(RdpBridge* bridge)
{
    if (!bridge)
        return -1;

    /* Enable Restricted Admin mode for pass-the-hash authentication */
    freerdp_settings_set_bool(bridge->settings, FreeRDP_RestrictedAdminModeRequired, TRUE);

    /* Also ensure NLA is enabled (required for PtH) */
    freerdp_settings_set_bool(bridge->settings, FreeRDP_NlaSecurity, TRUE);

    return 0;
}

int rdp_bridge_connect(RdpBridge* bridge)
{
    if (!bridge)
        return -1;

    /* Connect */
    if (!freerdp_connect(bridge->instance)) {
        UINT32 err = freerdp_get_last_error(bridge->context);
        snprintf(bridge->last_error, sizeof(bridge->last_error),
                 "freerdp_connect failed: %s (0x%08X)",
                 freerdp_get_last_error_string(err), err);
        return -1;
    }

    bridge->connected = 1;

    /* Wait for the connection to become active (logged in) */
    DWORD count;
    HANDLE handles[64];
    int attempts = 0;
    const int max_attempts = 600; /* ~60 seconds with 100ms waits */

    while (attempts < max_attempts) {
        /* Check if we should disconnect */
        if (freerdp_shall_disconnect_context(bridge->context)) {
            snprintf(bridge->last_error, sizeof(bridge->last_error),
                     "Connection terminated during login (disconnect reason: %s)",
                     freerdp_disconnect_reason_string(
                         freerdp_get_disconnect_ultimatum(bridge->context)));
            return -1;
        }

        /* Process pending events */
        count = freerdp_get_event_handles(bridge->context, handles, 64);
        if (count > 0) {
            WaitForMultipleObjects(count, handles, FALSE, 100);
        } else {
            Sleep(100);
        }

        if (!freerdp_check_event_handles(bridge->context)) {
            UINT32 err = freerdp_get_last_error(bridge->context);
            snprintf(bridge->last_error, sizeof(bridge->last_error),
                     "Event handling error: %s (0x%08X)",
                     freerdp_get_last_error_string(err), err);
            return -1;
        }

        /* Check if we're active now */
        if (freerdp_is_active_state(bridge->context)) {
            bridge->active = 1;

            /* Initialize GDI for framebuffer access (needed for screenshots) */
            if (!gdi_init(bridge->instance, PIXEL_FORMAT_BGRX32)) {
                snprintf(bridge->last_error, sizeof(bridge->last_error),
                         "gdi_init failed");
                freerdp_disconnect(bridge->instance);
                bridge->connected = 0;
                return -1;
            }

            /* Wait a moment for the desktop to settle */
            Sleep(3000);
            return 0;
        }

        attempts++;
    }

    snprintf(bridge->last_error, sizeof(bridge->last_error),
             "Timeout waiting for active connection state");
    return -1;
}

int rdp_bridge_is_active(RdpBridge* bridge)
{
    if (!bridge)
        return 0;
    return bridge->active;
}

int rdp_bridge_send_key(RdpBridge* bridge, int down, uint8_t scancode, int extended)
{
    if (!bridge || !bridge->active || !bridge->context)
        return -1;

    rdpInput* input = bridge->context->input;
    if (!input)
        return -1;

    UINT32 rdp_scancode = scancode;
    if (extended)
        rdp_scancode |= KBDEXT;

    BOOL ok = freerdp_input_send_keyboard_event_ex(input, (BOOL)down, FALSE, rdp_scancode);
    return ok ? 0 : -1;
}

int rdp_bridge_send_win_r(RdpBridge* bridge)
{
    if (!bridge)
        return -1;

    printf("[*] Sending Win+R to open Run dialog...\n");

    /* Windows key down */
    printf("[*]   Win key down\n");
    if (rdp_bridge_send_key(bridge, 1, 0x5B, 1) != 0) /* RDP_SCANCODE_LWIN */
        return -1;
    Sleep(50);

    /* R key down */
    printf("[*]   R key down\n");
    if (rdp_bridge_send_key(bridge, 1, 0x13, 0) != 0) /* RDP_SCANCODE_KEY_R */
        return -1;
    Sleep(50);

    /* R key up */
    printf("[*]   R key up\n");
    if (rdp_bridge_send_key(bridge, 0, 0x13, 0) != 0)
        return -1;
    Sleep(50);

    /* Windows key up */
    printf("[*]   Win key up\n");
    if (rdp_bridge_send_key(bridge, 0, 0x5B, 1) != 0)
        return -1;
    Sleep(500);

    printf("[+] Win+R sent successfully\n");
    return 0;
}

/* ── Screenshot support ─────────────────────────────────────────────── */

/// BMP file header (14 bytes) + DIB header (40 bytes) for 24-bit BI_RGB
#define BMP_HEADER_SIZE 54

/// Write a 24-bit BMP file from raw BGR pixel data.
/// Rows are assumed top-to-bottom (BMP stores bottom-up, we flip here).
static int write_bmp(const char* path,
                     int width, int height, int stride,
                     const BYTE* data, int bpp)
{
    if (width <= 0 || height <= 0 || !data)
        return -1;

    // BMP rows must be aligned to 4 bytes
    int row_bytes = width * 3;  // 24-bit: 3 bytes per pixel
    int pad = (4 - (row_bytes % 4)) % 4;
    int row_size = row_bytes + pad;
    int pixel_data_size = row_size * height;
    int file_size = BMP_HEADER_SIZE + pixel_data_size;

    FILE* fp = fopen(path, "wb");
    if (!fp)
        return -1;

    // BMP file header (14 bytes)
    unsigned char header[14] = {
        'B', 'M',             // signature
        0, 0, 0, 0,            // file size (filled below)
        0, 0, 0, 0,            // reserved
        BMP_HEADER_SIZE, 0, 0, 0  // pixel data offset
    };
    header[2] = (unsigned char)(file_size & 0xFF);
    header[3] = (unsigned char)((file_size >> 8) & 0xFF);
    header[4] = (unsigned char)((file_size >> 16) & 0xFF);
    header[5] = (unsigned char)((file_size >> 24) & 0xFF);

    fwrite(header, 1, 14, fp);

    // DIB header (BITMAPINFOHEADER, 40 bytes)
    unsigned char dib[40] = {0};
    dib[0] = 40;                                      // header size
    dib[4] = (unsigned char)(width & 0xFF);            // width
    dib[5] = (unsigned char)((width >> 8) & 0xFF);
    dib[6] = (unsigned char)((width >> 16) & 0xFF);
    dib[7] = (unsigned char)((width >> 24) & 0xFF);
    dib[8] = (unsigned char)(height & 0xFF);           // height (positive = bottom-up)
    dib[9] = (unsigned char)((height >> 8) & 0xFF);
    dib[10] = (unsigned char)((height >> 16) & 0xFF);
    dib[11] = (unsigned char)((height >> 24) & 0xFF);
    dib[12] = 1;                                       // planes
    dib[14] = 24;                                      // bits per pixel
    // BI_RGB (0) is already the default in the zeroed header

    fwrite(dib, 1, 40, fp);

    // Pixel data: write rows bottom-to-top, converting from source format
    unsigned char* row_buf = (unsigned char*)malloc(row_size);
    if (!row_buf) {
        fclose(fp);
        return -1;
    }
    memset(row_buf + row_bytes, 0, pad);  // zero the padding bytes

    for (int y = height - 1; y >= 0; y--) {
        const BYTE* src = data + (y * stride);
        if (bpp == 32) {
            // 32-bit BGRA/BGRX → 24-bit BGR
            for (int x = 0; x < width; x++) {
                row_buf[x * 3 + 0] = src[x * 4 + 0];  // B
                row_buf[x * 3 + 1] = src[x * 4 + 1];  // G
                row_buf[x * 3 + 2] = src[x * 4 + 2];  // R
            }
        } else {
            // 24-bit BGR → direct copy
            memcpy(row_buf, src, row_bytes);
        }
        fwrite(row_buf, 1, row_size, fp);
    }

    free(row_buf);
    fclose(fp);
    return 0;
}

/* GetTickCount work-alike using clock_gettime (Linux/POSIX).
   WinPR provides GetTickCount via <winpr/sysinfo.h>, but to keep
   things simple we use POSIX clocks directly. */
static DWORD get_ms_since_boot(void)
{
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (DWORD)(ts.tv_sec * 1000ULL + ts.tv_nsec / 1000000ULL);
}

int rdp_bridge_pump_events(RdpBridge* bridge, int timeout_ms)
{
    if (!bridge || !bridge->context || !bridge->connected)
        return -1;

    DWORD count;
    HANDLE handles[64];
    DWORD start = get_ms_since_boot();

    for (;;) {
        DWORD now = get_ms_since_boot();
        int elapsed = (int)(now - start);
        if (elapsed >= timeout_ms)
            break;

        if (freerdp_shall_disconnect_context(bridge->context))
            break;

        count = freerdp_get_event_handles(bridge->context, handles, 64);
        if (count > 0) {
            DWORD wait_ms = (DWORD)(timeout_ms - elapsed);
            if (wait_ms > 100) wait_ms = 100;
            WaitForMultipleObjects(count, handles, FALSE, wait_ms);
        } else {
            Sleep(50);
        }

        /* Always check and process events, regardless of whether
           WaitForMultipleObjects timed out. Without this, outgoing
           keyboard events are never actually transmitted, and the
           framebuffer (used for screenshots) never gets updated. */
        if (!freerdp_check_event_handles(bridge->context))
            break;
    }

    return 0;
}

int rdp_bridge_take_screenshot(RdpBridge* bridge, const char* path)
{
    if (!bridge || !bridge->context)
        return -1;

    rdpGdi* gdi = bridge->context->gdi;
    if (!gdi || !gdi->primary_buffer) {
        snprintf(bridge->last_error, sizeof(bridge->last_error),
                 "GDI not initialized, no framebuffer available");
        return -1;
    }

    // Determine bits-per-pixel from the pixel format
    int bpp = 32;
    UINT32 fmt = gdi->dstFormat;
    UINT32 fmt_bpp = (fmt & 0xFF);
    if (fmt_bpp == 24)
        bpp = 24;

    int ret = write_bmp(path, (int)gdi->width, (int)gdi->height,
                        (int)gdi->stride, gdi->primary_buffer, bpp);
    if (ret != 0) {
        snprintf(bridge->last_error, sizeof(bridge->last_error),
                 "Failed to write BMP to '%s'", path);
        return -1;
    }

    printf("[+] Screenshot saved to '%s' (%dx%d, %d bpp)\n",
           path, gdi->width, gdi->height, bpp);
    return 0;
}

int rdp_bridge_disconnect(RdpBridge* bridge)
{
    if (!bridge || !bridge->instance)
        return 0;

    if (bridge->connected) {
        freerdp_disconnect(bridge->instance);
        bridge->connected = 0;
    }
    bridge->active = 0;
    return 0;
}

void rdp_bridge_free(RdpBridge* bridge)
{
    if (!bridge)
        return;

    rdp_bridge_disconnect(bridge);

    if (bridge->context) {
        gdi_free(bridge->instance);
        freerdp_context_free(bridge->instance);
        bridge->context = NULL;
    }

    if (bridge->instance) {
        freerdp_free(bridge->instance);
        bridge->instance = NULL;
    }

    free(bridge);
}

const char* rdp_bridge_last_error(RdpBridge* bridge)
{
    if (!bridge)
        return "Null bridge pointer";
    return bridge->last_error;
}
