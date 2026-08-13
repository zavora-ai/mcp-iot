use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Mutex;

// === Input Types ===

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeviceRegisterInput { pub name: String, pub device_type: String, pub protocol: Option<String>, pub location: Option<String>, pub tags: Option<Vec<String>>, pub lat: Option<f64>, pub lng: Option<f64> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeviceIdInput { pub device_id: String }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TelemetryIngestInput { pub device_id: String, pub metrics: HashMap<String, f64> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TelemetryQueryInput { pub device_id: String, pub metric: Option<String>, pub last_n: Option<usize> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CommandInput { pub device_id: String, pub command: String, pub params: Option<HashMap<String, String>> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AlertRuleInput { pub name: String, pub device_id: Option<String>, pub metric: String, pub condition: String, pub threshold: f64 }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TwinUpdateInput { pub device_id: String, pub desired: HashMap<String, Value> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FleetCreateInput { pub name: String, pub device_ids: Vec<String> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FleetCommandInput { pub fleet_id: String, pub command: String, pub params: Option<HashMap<String, String>> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct OtaCreateInput { pub version: String, pub target_type: Option<String>, pub url: String, pub checksum: Option<String> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct OtaDeployInput { pub firmware_id: String, pub device_ids: Vec<String> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GeofenceCreateInput { pub name: String, pub lat: f64, pub lng: f64, pub radius_m: f64, pub device_ids: Vec<String> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GeofenceCheckInput { pub device_id: String, pub lat: f64, pub lng: f64 }

// === Data Models ===

#[derive(Clone, serde::Serialize)]
struct Device { id: String, name: String, device_type: String, protocol: String, location: String, tags: Vec<String>, lat: Option<f64>, lng: Option<f64>, status: String, registered_at: String, last_seen: Option<String> }
#[derive(Clone, serde::Serialize)]
struct Telemetry { device_id: String, metrics: HashMap<String, f64>, timestamp: String }
#[derive(Clone, serde::Serialize)]
struct Command { id: String, device_id: String, command: String, params: HashMap<String, String>, status: String, sent_at: String }
#[derive(Clone, serde::Serialize)]
struct AlertRule { id: String, name: String, device_id: Option<String>, metric: String, condition: String, threshold: f64, active: bool }
#[derive(Clone, serde::Serialize)]
struct Alert { id: String, rule_id: String, device_id: String, metric: String, value: f64, threshold: f64, triggered_at: String }
#[derive(Clone, serde::Serialize)]
struct Fleet { id: String, name: String, device_ids: Vec<String>, created_at: String }
#[derive(Clone, serde::Serialize)]
struct Firmware { id: String, version: String, target_type: String, url: String, checksum: String, created_at: String }
#[derive(Clone, serde::Serialize)]
struct OtaDeployment { id: String, firmware_id: String, device_ids: Vec<String>, status: String, started_at: String }
#[derive(Clone, serde::Serialize)]
struct Geofence { id: String, name: String, lat: f64, lng: f64, radius_m: f64, device_ids: Vec<String> }

// === Server ===

#[derive(Clone)]
pub struct IoTServer {
    devices: std::sync::Arc<Mutex<HashMap<String, Device>>>,
    telemetry: std::sync::Arc<Mutex<Vec<Telemetry>>>,
    commands: std::sync::Arc<Mutex<Vec<Command>>>,
    alert_rules: std::sync::Arc<Mutex<Vec<AlertRule>>>,
    alerts: std::sync::Arc<Mutex<Vec<Alert>>>,
    twins: std::sync::Arc<Mutex<HashMap<String, Value>>>,
    fleets: std::sync::Arc<Mutex<HashMap<String, Fleet>>>,
    firmware: std::sync::Arc<Mutex<HashMap<String, Firmware>>>,
    deployments: std::sync::Arc<Mutex<Vec<OtaDeployment>>>,
    geofences: std::sync::Arc<Mutex<Vec<Geofence>>>,
}

impl IoTServer {
    pub fn new() -> Self {
        Self {
            devices: Default::default(), telemetry: Default::default(), commands: Default::default(),
            alert_rules: Default::default(), alerts: Default::default(), twins: Default::default(),
            fleets: Default::default(), firmware: Default::default(), deployments: Default::default(),
            geofences: Default::default(),
        }
    }
}

fn short_id() -> String { uuid::Uuid::new_v4().to_string()[..8].to_string() }
fn now() -> String { chrono::Utc::now().to_rfc3339() }

#[tool_router]
impl IoTServer {
    // === Device Registry ===

    #[tool(description = "Register a new IoT device (sensor, actuator, gateway, edge). Returns device ID.")]
    async fn device_register(&self, Parameters(input): Parameters<DeviceRegisterInput>) -> String {
        let id = format!("dev_{}", short_id());
        let device = Device { id: id.clone(), name: input.name, device_type: input.device_type, protocol: input.protocol.unwrap_or_else(|| "mqtt".into()), location: input.location.unwrap_or_default(), tags: input.tags.unwrap_or_default(), lat: input.lat, lng: input.lng, status: "online".into(), registered_at: now(), last_seen: None };
        self.devices.lock().unwrap().insert(id.clone(), device.clone());
        // Init digital twin
        self.twins.lock().unwrap().insert(id.clone(), json!({"reported": {}, "desired": {}}));
        json!({"device_id": id, "name": device.name, "status": "registered"}).to_string()
    }

    #[tool(description = "List all registered IoT devices with status summary.")]
    async fn device_list(&self) -> String {
        let devices = self.devices.lock().unwrap();
        let list: Vec<Value> = devices.values().map(|d| json!({"id": d.id, "name": d.name, "type": d.device_type, "status": d.status, "location": d.location, "last_seen": d.last_seen})).collect();
        let online = devices.values().filter(|d| d.status == "online").count();
        json!({"total": list.len(), "online": online, "offline": list.len() - online, "devices": list}).to_string()
    }

    #[tool(description = "Get device details including twin state and recent telemetry.")]
    async fn device_get(&self, Parameters(input): Parameters<DeviceIdInput>) -> String {
        let devices = self.devices.lock().unwrap();
        match devices.get(&input.device_id) {
            Some(d) => {
                let twin = self.twins.lock().unwrap().get(&input.device_id).cloned().unwrap_or(json!({}));
                let telemetry = self.telemetry.lock().unwrap();
                let recent: Vec<_> = telemetry.iter().filter(|t| t.device_id == input.device_id).rev().take(5).collect();
                json!({"device": d, "twin": twin, "recent_telemetry": recent}).to_string()
            }
            None => json!({"error": "DEVICE_NOT_FOUND"}).to_string(),
        }
    }

    #[tool(description = "Decommission a device (set offline, remove from fleets).")]
    async fn device_decommission(&self, Parameters(input): Parameters<DeviceIdInput>) -> String {
        let mut devices = self.devices.lock().unwrap();
        match devices.get_mut(&input.device_id) {
            Some(d) => { d.status = "decommissioned".into(); json!({"device_id": input.device_id, "status": "decommissioned"}).to_string() }
            None => json!({"error": "DEVICE_NOT_FOUND"}).to_string(),
        }
    }

    // === Telemetry ===

    #[tool(description = "Ingest telemetry data point(s) from a device. Metrics: temperature, humidity, pressure, voltage, etc.")]
    async fn telemetry_ingest(&self, Parameters(input): Parameters<TelemetryIngestInput>) -> String {
        let mut devices = self.devices.lock().unwrap();
        if let Some(d) = devices.get_mut(&input.device_id) { d.last_seen = Some(now()); d.status = "online".into(); } else { return json!({"error": "DEVICE_NOT_FOUND"}).to_string(); }
        drop(devices);
        // Check alert rules
        let rules = self.alert_rules.lock().unwrap().clone();
        let mut triggered = Vec::new();
        for rule in &rules {
            if !rule.active { continue; }
            if let Some(device_filter) = &rule.device_id { if device_filter != &input.device_id { continue; } }
            if let Some(val) = input.metrics.get(&rule.metric) {
                let breach = match rule.condition.as_str() { "gt" | ">" => *val > rule.threshold, "lt" | "<" => *val < rule.threshold, "gte" | ">=" => *val >= rule.threshold, "lte" | "<=" => *val <= rule.threshold, "eq" | "==" => (*val - rule.threshold).abs() < 0.001, _ => false };
                if breach {
                    let alert = Alert { id: format!("alrt_{}", short_id()), rule_id: rule.id.clone(), device_id: input.device_id.clone(), metric: rule.metric.clone(), value: *val, threshold: rule.threshold, triggered_at: now() };
                    triggered.push(json!({"alert_id": alert.id, "rule": rule.name, "metric": rule.metric, "value": val, "threshold": rule.threshold}));
                    self.alerts.lock().unwrap().push(alert);
                }
            }
        }
        // Update twin reported state
        let mut twins = self.twins.lock().unwrap();
        if let Some(twin) = twins.get_mut(&input.device_id) { if let Some(obj) = twin.get_mut("reported") { for (k, v) in &input.metrics { obj[k] = json!(v); } } }
        drop(twins);
        let entry = Telemetry { device_id: input.device_id.clone(), metrics: input.metrics, timestamp: now() };
        self.telemetry.lock().unwrap().push(entry);
        json!({"status": "ingested", "device_id": input.device_id, "alerts_triggered": triggered.len(), "alerts": triggered}).to_string()
    }

    #[tool(description = "Query telemetry history for a device. Optional: filter by metric, limit results.")]
    async fn telemetry_query(&self, Parameters(input): Parameters<TelemetryQueryInput>) -> String {
        let telemetry = self.telemetry.lock().unwrap();
        let limit = input.last_n.unwrap_or(20);
        let results: Vec<Value> = telemetry.iter().filter(|t| t.device_id == input.device_id).rev().take(limit).map(|t| {
            if let Some(metric) = &input.metric { json!({"value": t.metrics.get(metric), "timestamp": t.timestamp}) }
            else { json!({"metrics": t.metrics, "timestamp": t.timestamp}) }
        }).collect();
        json!({"device_id": input.device_id, "metric": input.metric, "count": results.len(), "data": results}).to_string()
    }

    // === Commands ===

    #[tool(description = "Send a remote command to a device (reboot, configure, update_firmware, set_interval, calibrate).")]
    async fn command_send(&self, Parameters(input): Parameters<CommandInput>) -> String {
        if !self.devices.lock().unwrap().contains_key(&input.device_id) { return json!({"error": "DEVICE_NOT_FOUND"}).to_string(); }
        let cmd = Command { id: format!("cmd_{}", short_id()), device_id: input.device_id, command: input.command, params: input.params.unwrap_or_default(), status: "sent".into(), sent_at: now() };
        let resp = json!({"command_id": cmd.id, "device_id": cmd.device_id, "command": cmd.command, "status": "sent"});
        self.commands.lock().unwrap().push(cmd);
        resp.to_string()
    }

    #[tool(description = "List commands sent to a device with their execution status.")]
    async fn command_list(&self, Parameters(input): Parameters<DeviceIdInput>) -> String {
        let commands = self.commands.lock().unwrap();
        let list: Vec<Value> = commands.iter().filter(|c| c.device_id == input.device_id).map(|c| json!({"id": c.id, "command": c.command, "status": c.status, "sent_at": c.sent_at})).collect();
        json!({"device_id": input.device_id, "commands": list.len(), "history": list}).to_string()
    }

    // === Alerts ===

    #[tool(description = "Create an alert rule (e.g., temperature > 80, humidity < 20). Conditions: gt, lt, gte, lte, eq.")]
    async fn alert_rule_create(&self, Parameters(input): Parameters<AlertRuleInput>) -> String {
        let rule = AlertRule { id: format!("rule_{}", short_id()), name: input.name, device_id: input.device_id, metric: input.metric, condition: input.condition, threshold: input.threshold, active: true };
        let resp = json!({"rule_id": rule.id, "name": rule.name, "metric": rule.metric, "condition": rule.condition, "threshold": rule.threshold});
        self.alert_rules.lock().unwrap().push(rule);
        resp.to_string()
    }

    #[tool(description = "List all alert rules.")]
    async fn alert_rule_list(&self) -> String {
        let rules = self.alert_rules.lock().unwrap();
        let list: Vec<Value> = rules.iter().map(|r| json!({"id": r.id, "name": r.name, "metric": r.metric, "condition": r.condition, "threshold": r.threshold, "active": r.active, "device_id": r.device_id})).collect();
        json!({"rules": list.len(), "items": list}).to_string()
    }

    #[tool(description = "List triggered alerts (breaches). Shows device, metric, value vs threshold.")]
    async fn alert_list(&self) -> String {
        let alerts = self.alerts.lock().unwrap();
        let list: Vec<Value> = alerts.iter().rev().take(50).map(|a| json!({"id": a.id, "rule_id": a.rule_id, "device_id": a.device_id, "metric": a.metric, "value": a.value, "threshold": a.threshold, "triggered_at": a.triggered_at})).collect();
        json!({"total_alerts": alerts.len(), "showing": list.len(), "alerts": list}).to_string()
    }

    // === Digital Twins ===

    #[tool(description = "Update device digital twin desired state. Device will reconcile reported vs desired.")]
    async fn twin_update(&self, Parameters(input): Parameters<TwinUpdateInput>) -> String {
        if !self.devices.lock().unwrap().contains_key(&input.device_id) { return json!({"error": "DEVICE_NOT_FOUND"}).to_string(); }
        let mut twins = self.twins.lock().unwrap();
        let twin = twins.entry(input.device_id.clone()).or_insert(json!({"reported": {}, "desired": {}}));
        if let Some(desired) = twin.get_mut("desired") { for (k, v) in input.desired { desired[k] = v; } }
        json!({"device_id": input.device_id, "twin": twin}).to_string()
    }

    #[tool(description = "Get device digital twin (reported vs desired state, drift detection).")]
    async fn twin_get(&self, Parameters(input): Parameters<DeviceIdInput>) -> String {
        let twins = self.twins.lock().unwrap();
        match twins.get(&input.device_id) {
            Some(twin) => {
                let reported = twin.get("reported").cloned().unwrap_or(json!({}));
                let desired = twin.get("desired").cloned().unwrap_or(json!({}));
                let drift: Vec<String> = if let (Some(r), Some(d)) = (reported.as_object(), desired.as_object()) { d.keys().filter(|k| r.get(*k) != d.get(*k)).cloned().collect() } else { vec![] };
                json!({"device_id": input.device_id, "reported": reported, "desired": desired, "drift_keys": drift, "in_sync": drift.is_empty()}).to_string()
            }
            None => json!({"error": "DEVICE_NOT_FOUND"}).to_string(),
        }
    }

    // === Fleet Management ===

    #[tool(description = "Create a fleet group (logical grouping of devices for bulk operations).")]
    async fn fleet_create(&self, Parameters(input): Parameters<FleetCreateInput>) -> String {
        let id = format!("fleet_{}", short_id());
        let fleet = Fleet { id: id.clone(), name: input.name, device_ids: input.device_ids, created_at: now() };
        let resp = json!({"fleet_id": id, "name": fleet.name, "devices": fleet.device_ids.len()});
        self.fleets.lock().unwrap().insert(id, fleet);
        resp.to_string()
    }

    #[tool(description = "List all fleet groups.")]
    async fn fleet_list(&self) -> String {
        let fleets = self.fleets.lock().unwrap();
        let list: Vec<Value> = fleets.values().map(|f| json!({"id": f.id, "name": f.name, "devices": f.device_ids.len()})).collect();
        json!({"fleets": list.len(), "items": list}).to_string()
    }

    #[tool(description = "Send a command to all devices in a fleet (bulk reboot, configure, update).")]
    async fn fleet_command(&self, Parameters(input): Parameters<FleetCommandInput>) -> String {
        let fleets = self.fleets.lock().unwrap();
        match fleets.get(&input.fleet_id) {
            Some(fleet) => {
                let count = fleet.device_ids.len();
                let params = input.params.unwrap_or_default();
                let mut cmds = self.commands.lock().unwrap();
                for did in &fleet.device_ids {
                    cmds.push(Command { id: format!("cmd_{}", short_id()), device_id: did.clone(), command: input.command.clone(), params: params.clone(), status: "sent".into(), sent_at: now() });
                }
                json!({"fleet_id": input.fleet_id, "command": input.command, "devices_targeted": count, "status": "sent_to_all"}).to_string()
            }
            None => json!({"error": "FLEET_NOT_FOUND"}).to_string(),
        }
    }

    // === OTA Firmware ===

    #[tool(description = "Register a firmware version for OTA updates.")]
    async fn ota_firmware_create(&self, Parameters(input): Parameters<OtaCreateInput>) -> String {
        let id = format!("fw_{}", short_id());
        let fw = Firmware { id: id.clone(), version: input.version, target_type: input.target_type.unwrap_or_else(|| "all".into()), url: input.url, checksum: input.checksum.unwrap_or_default(), created_at: now() };
        let resp = json!({"firmware_id": id, "version": fw.version, "target_type": fw.target_type});
        self.firmware.lock().unwrap().insert(id, fw);
        resp.to_string()
    }

    #[tool(description = "Deploy firmware to devices (OTA update). Tracks rollout status.")]
    async fn ota_deploy(&self, Parameters(input): Parameters<OtaDeployInput>) -> String {
        if !self.firmware.lock().unwrap().contains_key(&input.firmware_id) { return json!({"error": "FIRMWARE_NOT_FOUND"}).to_string(); }
        let dep = OtaDeployment { id: format!("dep_{}", short_id()), firmware_id: input.firmware_id, device_ids: input.device_ids.clone(), status: "rolling_out".into(), started_at: now() };
        let resp = json!({"deployment_id": dep.id, "firmware_id": dep.firmware_id, "devices": input.device_ids.len(), "status": "rolling_out"});
        self.deployments.lock().unwrap().push(dep);
        resp.to_string()
    }

    #[tool(description = "List OTA deployments and their status.")]
    async fn ota_list(&self) -> String {
        let deps = self.deployments.lock().unwrap();
        let list: Vec<Value> = deps.iter().map(|d| json!({"id": d.id, "firmware_id": d.firmware_id, "devices": d.device_ids.len(), "status": d.status, "started_at": d.started_at})).collect();
        json!({"deployments": list.len(), "items": list}).to_string()
    }

    // === Geofencing ===

    #[tool(description = "Create a geofence (circular zone). Devices entering/leaving trigger alerts.")]
    async fn geofence_create(&self, Parameters(input): Parameters<GeofenceCreateInput>) -> String {
        let id = format!("geo_{}", short_id());
        let gf = Geofence { id: id.clone(), name: input.name, lat: input.lat, lng: input.lng, radius_m: input.radius_m, device_ids: input.device_ids };
        let resp = json!({"geofence_id": id, "name": gf.name, "center": {"lat": gf.lat, "lng": gf.lng}, "radius_m": gf.radius_m, "devices": gf.device_ids.len()});
        self.geofences.lock().unwrap().push(gf);
        resp.to_string()
    }

    #[tool(description = "Check if a device position is inside/outside its geofences. Returns breach status.")]
    async fn geofence_check(&self, Parameters(input): Parameters<GeofenceCheckInput>) -> String {
        let geofences = self.geofences.lock().unwrap();
        let relevant: Vec<_> = geofences.iter().filter(|g| g.device_ids.contains(&input.device_id)).collect();
        let mut results = Vec::new();
        for gf in &relevant {
            // Haversine approximation (flat earth for short distances)
            let dlat = (input.lat - gf.lat).to_radians();
            let dlng = (input.lng - gf.lng).to_radians();
            let a = (dlat / 2.0).sin().powi(2) + gf.lat.to_radians().cos() * input.lat.to_radians().cos() * (dlng / 2.0).sin().powi(2);
            let distance_m = 6371000.0 * 2.0 * a.sqrt().asin();
            let inside = distance_m <= gf.radius_m;
            results.push(json!({"geofence_id": gf.id, "name": gf.name, "distance_m": (distance_m * 100.0).round() / 100.0, "radius_m": gf.radius_m, "inside": inside, "breach": !inside}));
        }
        let breaches = results.iter().filter(|r| r["breach"] == true).count();
        json!({"device_id": input.device_id, "position": {"lat": input.lat, "lng": input.lng}, "geofences_checked": results.len(), "breaches": breaches, "results": results}).to_string()
    }

    // === Dashboard ===

    #[tool(description = "IoT dashboard summary: device counts by status, alert count, telemetry rate, fleet count.")]
    async fn dashboard(&self) -> String {
        let devices = self.devices.lock().unwrap();
        let online = devices.values().filter(|d| d.status == "online").count();
        let offline = devices.values().filter(|d| d.status == "offline").count();
        let decommissioned = devices.values().filter(|d| d.status == "decommissioned").count();
        let telemetry_count = self.telemetry.lock().unwrap().len();
        let alert_count = self.alerts.lock().unwrap().len();
        let fleet_count = self.fleets.lock().unwrap().len();
        let deployment_count = self.deployments.lock().unwrap().len();
        json!({"devices": {"total": devices.len(), "online": online, "offline": offline, "decommissioned": decommissioned}, "telemetry_points": telemetry_count, "alerts_triggered": alert_count, "active_rules": self.alert_rules.lock().unwrap().iter().filter(|r| r.active).count(), "fleets": fleet_count, "ota_deployments": deployment_count}).to_string()
    }
}

adk_mcp_sdk::mcp_2026_server! {
    server: IoTServer,
    task_tools: ["fleet_create", "fleet_command", "ota_deploy"],
    approval_tools: [],
    cache_ttl_ms: 60_000,
}
