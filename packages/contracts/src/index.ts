/**
 * @bambu-monitor/contracts
 *
 * Public interface for frontend applications to consume API and telemetry types.
 *
 * @security
 * - Only TYPE exports from @bambu-monitor/api (no runtime code)
 *
 * @usage Frontend apps (ui) should import from this package:
 * ```ts
 * import type { App } from "@bambu-monitor/contracts";
 * ```
 */

// Re-export API types (type-only - App for Elysia app type for Eden)
export type {
  App,
  DeviceCommand,
  DeviceConfig,
  DeviceConnection,
  DeviceCreateInput,
  DeviceProbeResult,
  DeviceSnapshot,
  DeviceTelemetry,
} from "@bambu-monitor/api";
