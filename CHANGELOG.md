# Changelog

## [1.1.0] - 2026-08-13

### Changed
- Upgraded to rmcp 3.1.2 and raised the minimum supported Rust version to 1.94.1.
- Added MCP 2026-07-28 stateless request handling while retaining MCP 2025-11-25 initialization compatibility.

### Added
- Per-request identity and protocol metadata, on-demand discovery/cache hints, and the configured Tasks and sealed MRTR approval policies.

## [1.0.0] - 2026-05-27

### Added
- `device_register` / `device_list` / `device_get` / `device_decommission` — device registry
- `telemetry_ingest` — ingest metrics with auto-alert triggering
- `telemetry_query` — query telemetry history with metric filter
- `command_send` / `command_list` — remote command execution
- `alert_rule_create` / `alert_rule_list` / `alert_list` — alert rules with conditions (gt, lt, gte, lte, eq)
- `twin_update` / `twin_get` — digital twin with drift detection
- `fleet_create` / `fleet_list` / `fleet_command` — fleet management and bulk operations
- `ota_firmware_create` / `ota_deploy` / `ota_list` — OTA firmware management
- `geofence_create` / `geofence_check` — geofencing with haversine breach detection
- `dashboard` — IoT summary dashboard
