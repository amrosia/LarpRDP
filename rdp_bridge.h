// SPDX-License-Identifier: AGPL-3.0

#ifndef RDP_BRIDGE_H
#define RDP_BRIDGE_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Opaque handle to the RDP connection state */
typedef struct RdpBridge RdpBridge;

/* Create a new RDP bridge instance.
 * Returns NULL on failure.
 */
RdpBridge* rdp_bridge_create(void);

/* Configure connection parameters.
 * Must be called before rdp_bridge_connect().
 * Returns 0 on success, -1 on failure.
 */
int rdp_bridge_set_server(RdpBridge* bridge, const char* server);
int rdp_bridge_set_credentials(RdpBridge* bridge,
                               const char* username,
                               const char* password,
                               const char* domain);

/* Enable pass-the-hash authentication.
 * Sets the password as an NTLM hash and enables Restricted Admin mode.
 * Must be called after set_credentials() and before connect().
 */
int rdp_bridge_enable_pth(RdpBridge* bridge);

/* Initiate the RDP connection and wait for active session.
 * Blocks until connected or error.
 * Returns 0 on success, -1 on failure.
 */
int rdp_bridge_connect(RdpBridge* bridge);

/* Check if the connection is active (logged in and ready for input).
 * Returns 1 if active, 0 if not.
 */
int rdp_bridge_is_active(RdpBridge* bridge);

/* Send a keyboard event.
 * down: 1 for key press, 0 for key release
 * scancode: RDP scancode (0-255)
 * extended: 1 if this is an extended key (e.g., Win, Right Alt, arrows, etc.)
 * Returns 0 on success, -1 on failure.
 */
int rdp_bridge_send_key(RdpBridge* bridge, int down, uint8_t scancode, int extended);

/* Convenience: send Win+R (opens the Run dialog).
 * Returns 0 on success, -1 on failure.
 */
int rdp_bridge_send_win_r(RdpBridge* bridge);

/* Pump pending RDP events, waiting up to timeout_ms milliseconds.
 * Processes incoming bitmap updates and keeps the framebuffer current.
 * Should be called instead of Sleep() to keep the connection alive.
 */
int rdp_bridge_pump_events(RdpBridge* bridge, int timeout_ms);

/* Take a screenshot of the current RDP session.
 * Writes a BMP file to the given path.
 * Returns 0 on success, -1 on failure.
 */
int rdp_bridge_take_screenshot(RdpBridge* bridge, const char* path);

/* Disconnect the RDP session.
 * Returns 0 on success.
 */
int rdp_bridge_disconnect(RdpBridge* bridge);

/* Free all resources associated with the bridge.
 */
void rdp_bridge_free(RdpBridge* bridge);

/* Get the last error message.
 * Returns a static string (do not free).
 */
const char* rdp_bridge_last_error(RdpBridge* bridge);

#ifdef __cplusplus
}
#endif

#endif /* RDP_BRIDGE_H */
