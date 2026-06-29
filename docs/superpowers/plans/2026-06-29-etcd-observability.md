# Etcd Observability Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add etcd endpoint health details, key revision lookup/history, and SSE-based key/prefix watch to EtcdPilot.

**Architecture:** Extend the current Axum backend in small slices, keeping etcd access inside `backend/src/etcd.rs` and HTTP concerns inside `backend/src/handlers.rs`. Enhance existing Vue views rather than adding new routes: `ClustersView.vue` gets structured endpoint status, and `KeysView.vue` gets a key detail panel with Details, History, and Watch tabs.

**Tech Stack:** Rust 2021, Axum 0.7, etcd-client 0.14, Tokio, serde/serde_json, Vue 3 Composition API, Axios, browser `EventSource`.

---

## File Map

- Modify `backend/src/models.rs`: add serializable DTOs for endpoint status, key history, and watch events.
- Modify `backend/src/etcd.rs`: add endpoint-by-endpoint status probing, revision-aware key reads, bounded history probing, and watch stream creation.
- Modify `backend/src/handlers.rs`: add routes and handlers for endpoint status, key history, revision query, and SSE watch.
- Modify `frontend/src/views/ClustersView.vue`: render structured endpoint status cards.
- Modify `frontend/src/views/KeysView.vue`: add selected key details, revision history, and watch event stream UI.
- Modify `frontend/src/i18n/locales/en-US.ts` and `frontend/src/i18n/locales/zh-CN.ts`: add labels for new UI states.
- Optional modify `README.en.md` and `README.zh-CN.md`: document watch/revision/endpoint status after implementation.

## Task 1: Backend Endpoint Status DTOs And API

**Files:**
- Modify: `backend/src/models.rs`
- Modify: `backend/src/etcd.rs`
- Modify: `backend/src/handlers.rs`

- [ ] **Step 1: Add endpoint status models**

Add to `backend/src/models.rs` after `ClusterInfo`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointStatusItem {
    pub endpoint: String,
    pub reachable: bool,
    pub version: Option<String>,
    pub leader: Option<u64>,
    pub raft_term: Option<u64>,
    pub raft_index: Option<u64>,
    pub raft_applied_index: Option<u64>,
    pub raft_used_db_size: Option<i64>,
    pub db_size: Option<i64>,
    pub errors: Vec<String>,
    pub is_learner: Option<bool>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointStatusResponse {
    pub cluster_id: String,
    pub endpoints: Vec<EndpointStatusItem>,
}
```

- [ ] **Step 2: Add endpoint status implementation**

Update imports in `backend/src/etcd.rs`:

```rust
use crate::{
    db,
    error::{AppError, Result},
    models::{EndpointStatusItem, EndpointStatusResponse, KvItem},
    AppState,
};
```

Add this function after `status`:

```rust
pub async fn endpoint_statuses(cluster: &ResolvedCluster) -> Result<EndpointStatusResponse> {
    let endpoints = normalize_endpoints(&cluster.endpoints);
    if endpoints.is_empty() {
        return Err(AppError::Validation("cluster has no endpoint".to_string()));
    }

    let mut items = Vec::with_capacity(endpoints.len());
    for endpoint in endpoints {
        let result = async {
            let client = connect_client(&[endpoint.clone()]).await?;
            let mut maintenance = client.maintenance_client();
            let response = timeout(Duration::from_secs(3), maintenance.status())
                .await
                .map_err(|_| AppError::Validation("etcd status timeout".to_string()))?
                .map_err(|err| AppError::Internal(format!("etcd status failed: {err}")))?;

            Ok::<EndpointStatusItem, AppError>(EndpointStatusItem {
                endpoint: endpoint.clone(),
                reachable: true,
                version: Some(response.version().to_string()),
                leader: Some(response.leader()),
                raft_term: Some(response.raft_term()),
                raft_index: Some(response.raft_index()),
                raft_applied_index: Some(response.raft_applied_index()),
                raft_used_db_size: Some(response.raft_used_db_size()),
                db_size: Some(response.db_size()),
                errors: response.errors().to_vec(),
                is_learner: Some(response.is_learner()),
                error: None,
            })
        }
        .await;

        items.push(result.unwrap_or_else(|err| EndpointStatusItem {
            endpoint,
            reachable: false,
            version: None,
            leader: None,
            raft_term: None,
            raft_index: None,
            raft_applied_index: None,
            raft_used_db_size: None,
            db_size: None,
            errors: Vec::new(),
            error: Some(err.to_string()),
            is_learner: None,
        }));
    }

    Ok(EndpointStatusResponse {
        cluster_id: cluster.cluster_id.clone(),
        endpoints: items,
    })
}
```

- [ ] **Step 3: Wire the HTTP route**

Update `backend/src/handlers.rs` route list:

```rust
.route("/api/clusters/:id/endpoints/status", get(cluster_endpoint_statuses))
```

Place it after `/api/clusters/:id/status`.

Add handler after `cluster_status`:

```rust
async fn cluster_endpoint_statuses(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<crate::models::EndpointStatusResponse>> {
    let user = auth::require_user(&headers, &state.config)?;
    require_permission(&user, "cluster:read")?;
    let cluster = etcd::resolve_cluster_by_id(&state, &id).await?;
    let status = etcd::endpoint_statuses(&cluster).await?;
    Ok(Json(status))
}
```

- [ ] **Step 4: Verify backend compiles**

Run:

```powershell
cargo check
```

Expected: `Finished dev profile` with no errors.

- [ ] **Step 5: Commit endpoint status backend**

```powershell
git add backend/src/models.rs backend/src/etcd.rs backend/src/handlers.rs
git commit -m "feat: add endpoint status API"
```

## Task 2: Frontend Cluster Endpoint Status UI

**Files:**
- Modify: `frontend/src/views/ClustersView.vue`
- Modify: `frontend/src/i18n/locales/en-US.ts`
- Modify: `frontend/src/i18n/locales/zh-CN.ts`

- [ ] **Step 1: Add frontend types and state**

In `frontend/src/views/ClustersView.vue`, add interfaces near existing types:

```ts
interface EndpointStatusItem {
  endpoint: string;
  reachable: boolean;
  version?: string;
  leader?: number;
  raft_term?: number;
  raft_index?: number;
  raft_applied_index?: number;
  raft_used_db_size?: number;
  db_size?: number;
  errors: string[];
  is_learner?: boolean;
  error?: string;
}

interface EndpointStatusResponse {
  cluster_id: string;
  endpoints: EndpointStatusItem[];
}
```

Add state near `clusterStatus`:

```ts
const endpointStatuses = ref<Record<string, EndpointStatusItem[]>>({});
```

- [ ] **Step 2: Fetch endpoint statuses with existing status load**

Find `loadStatus`. Replace the body with:

```ts
const loadStatus = async (id: string) => {
  markBusy(id, "status");
  try {
    const [statusResp, endpointResp] = await Promise.all([
      api.get(`/clusters/${id}/status`),
      api.get<EndpointStatusResponse>(`/clusters/${id}/endpoints/status`),
    ]);
    clusterStatus.value[id] = statusResp.data;
    endpointStatuses.value[id] = endpointResp.data.endpoints || [];
  } catch (err: any) {
    error.value = err.message || t("clusters.loadFailed");
  } finally {
    markBusy(id, "");
  }
};
```

- [ ] **Step 3: Render structured endpoint cards**

Inside the cluster card where raw status currently renders, add this block before the raw JSON/technical details:

```vue
<div v-if="endpointStatuses[cluster.id]?.length" class="endpoint-status-list">
  <article
    v-for="endpoint in endpointStatuses[cluster.id]"
    :key="endpoint.endpoint"
    class="endpoint-status-card"
    :class="{ reachable: endpoint.reachable, failed: !endpoint.reachable }"
  >
    <header>
      <span class="status-dot" :class="endpoint.reachable ? 'online' : 'offline'"></span>
      <strong class="mono">{{ endpoint.endpoint }}</strong>
    </header>
    <div class="endpoint-metrics">
      <span>{{ t("clusters.version") }}: {{ endpoint.version || t("common.notSet") }}</span>
      <span>{{ t("clusters.leader") }}: {{ endpoint.leader ?? t("common.notSet") }}</span>
      <span>{{ t("clusters.raftIndex") }}: {{ endpoint.raft_index ?? t("common.notSet") }}</span>
      <span>{{ t("clusters.dbSize") }}: {{ formatBytes(endpoint.db_size) }}</span>
    </div>
    <p v-if="endpoint.error" class="endpoint-error">{{ endpoint.error }}</p>
  </article>
</div>
```

Add helper:

```ts
const formatBytes = (value?: number) => {
  if (!value || value < 0) return t("common.notSet");
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`;
  if (value < 1024 * 1024 * 1024) return `${(value / 1024 / 1024).toFixed(1)} MB`;
  return `${(value / 1024 / 1024 / 1024).toFixed(1)} GB`;
};
```

- [ ] **Step 4: Add CSS for endpoint cards**

Add scoped styles:

```css
.endpoint-status-list {
  display: grid;
  gap: 10px;
  margin-top: 12px;
}

.endpoint-status-card {
  display: grid;
  gap: 8px;
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: rgba(0, 0, 0, 0.18);
}

.endpoint-status-card.reachable {
  border-color: rgba(74, 222, 128, 0.22);
}

.endpoint-status-card.failed {
  border-color: rgba(248, 113, 113, 0.22);
}

.endpoint-status-card header {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.endpoint-status-card header strong {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.endpoint-metrics {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: 6px 12px;
  color: var(--muted);
  font-size: 12px;
}

.endpoint-error {
  margin: 0;
  color: var(--danger);
  font-size: 12px;
  line-height: 1.5;
  word-break: break-word;
}
```

- [ ] **Step 5: Add i18n keys**

In both locale files under `clusters`, add:

```ts
version: "Version",
leader: "Leader",
raftIndex: "Raft index",
dbSize: "DB size",
```

Use Chinese equivalents in `zh-CN.ts`:

```ts
version: "版本",
leader: "Leader",
raftIndex: "Raft 索引",
dbSize: "数据库大小",
```

- [ ] **Step 6: Verify frontend build**

Run:

```powershell
npm run build
```

Expected: Vite build succeeds. Existing chunk-size warning is acceptable.

- [ ] **Step 7: Commit cluster UI**

```powershell
git add frontend/src/views/ClustersView.vue frontend/src/i18n/locales/en-US.ts frontend/src/i18n/locales/zh-CN.ts
git commit -m "feat: show endpoint health details"
```

## Task 3: Backend Revision Read And History API

**Files:**
- Modify: `backend/src/models.rs`
- Modify: `backend/src/etcd.rs`
- Modify: `backend/src/handlers.rs`

- [ ] **Step 1: Add key history models**

Add to `backend/src/models.rs` after `KvItem`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvHistoryResponse {
    pub key: String,
    pub compacted: bool,
    pub items: Vec<KvItem>,
}
```

- [ ] **Step 2: Extend query DTOs**

In `backend/src/handlers.rs`, replace `KvQuery` with:

```rust
#[derive(Deserialize)]
pub struct KvQuery {
    pub key: Option<String>,
    pub revision: Option<i64>,
}

#[derive(Deserialize)]
pub struct KvHistoryQuery {
    pub key: String,
    pub limit: Option<usize>,
}
```

- [ ] **Step 3: Implement revision read**

In `backend/src/etcd.rs`, replace `get_kv_item` with:

```rust
pub async fn get_kv_item(
    cluster: &ResolvedCluster,
    key: String,
    revision: Option<i64>,
) -> Result<Option<KvItem>> {
    let key = key.trim().to_string();
    if key.is_empty() {
        return Ok(None);
    }
    let options = revision
        .filter(|value| *value > 0)
        .map(|value| GetOptions::new().with_revision(value));
    let mut kvs = fetch_kv(cluster, key, options).await?;
    Ok(kvs.pop())
}
```

Update `get_kv_item` handler call:

```rust
let item = etcd::get_kv_item(&cluster, key, query.revision).await?;
```

- [ ] **Step 4: Implement bounded history**

In `backend/src/etcd.rs`, import `KvHistoryResponse`:

```rust
models::{EndpointStatusItem, EndpointStatusResponse, KvHistoryResponse, KvItem},
```

Add after `get_kv_item`:

```rust
pub async fn kv_history(
    cluster: &ResolvedCluster,
    key: String,
    limit: usize,
) -> Result<KvHistoryResponse> {
    let key = key.trim().to_string();
    if key.is_empty() {
        return Err(AppError::Validation("key is required".to_string()));
    }

    let current = get_kv_item(cluster, key.clone(), None).await?;
    let Some(current) = current else {
        return Ok(KvHistoryResponse {
            key,
            compacted: false,
            items: Vec::new(),
        });
    };

    let mut items = vec![current.clone()];
    let mut compacted = false;
    let mut revision = current.mod_revision - 1;
    let target = limit.clamp(1, 50);

    while revision > 0 && items.len() < target {
        match get_kv_item(cluster, key.clone(), Some(revision)).await {
            Ok(Some(item)) => {
                let is_duplicate = items
                    .last()
                    .map(|last| last.value == item.value && last.mod_revision == item.mod_revision)
                    .unwrap_or(false);
                if !is_duplicate {
                    items.push(item.clone());
                }
                revision = item.mod_revision.saturating_sub(1);
            }
            Ok(None) => break,
            Err(err) if err.to_string().to_lowercase().contains("compacted") => {
                compacted = true;
                break;
            }
            Err(err) => return Err(err),
        }
    }

    Ok(KvHistoryResponse {
        key,
        compacted,
        items,
    })
}
```

- [ ] **Step 5: Add history HTTP route**

In route list:

```rust
.route("/api/clusters/:id/kv/history", get(get_kv_history))
```

Add handler after `get_kv_item`:

```rust
async fn get_kv_history(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<KvHistoryQuery>,
) -> Result<Json<crate::models::KvHistoryResponse>> {
    let user = auth::require_user(&headers, &state.config)?;
    require_permission(&user, "key:read")?;
    let cluster = etcd::resolve_cluster_by_id(&state, &id).await?;
    let history = etcd::kv_history(&cluster, query.key, query.limit.unwrap_or(20)).await?;
    Ok(Json(history))
}
```

- [ ] **Step 6: Verify backend compiles**

Run:

```powershell
cargo check
```

Expected: `Finished dev profile` with no errors.

- [ ] **Step 7: Commit revision/history backend**

```powershell
git add backend/src/models.rs backend/src/etcd.rs backend/src/handlers.rs
git commit -m "feat: add key revision history API"
```

## Task 4: Frontend Key Detail And History UI

**Files:**
- Modify: `frontend/src/views/KeysView.vue`
- Modify: `frontend/src/i18n/locales/en-US.ts`
- Modify: `frontend/src/i18n/locales/zh-CN.ts`

- [ ] **Step 1: Add state**

In `KeysView.vue`, add:

```ts
type KeyDetailTab = "details" | "history" | "watch";

interface KvHistoryResponse {
  key: string;
  compacted: boolean;
  items: KvItem[];
}

const selectedKey = ref<KvItem | null>(null);
const detailTab = ref<KeyDetailTab>("details");
const history = ref<KvItem[]>([]);
const historyCompacted = ref(false);
const historyLoading = ref(false);
const historyError = ref("");
const revisionInput = ref("");
const revisionItem = ref<KvItem | null>(null);
```

- [ ] **Step 2: Select key from the list**

Where key rows/cards are clicked, call:

```ts
const selectKey = (row: KvItem) => {
  selectedKey.value = row;
  detailTab.value = "details";
  revisionInput.value = String(row.revision);
  revisionItem.value = null;
  history.value = [];
  historyCompacted.value = false;
  historyError.value = "";
};
```

Attach `@click="selectKey(row)"` to the key row/card root.

- [ ] **Step 3: Load history and revision**

Add:

```ts
const loadHistory = async () => {
  if (!clusterId.value || !selectedKey.value) return;
  historyLoading.value = true;
  historyError.value = "";
  try {
    const resp = await api.get<KvHistoryResponse>(`/clusters/${clusterId.value}/kv/history`, {
      params: { key: selectedKey.value.key, limit: 20 },
    });
    history.value = resp.data.items || [];
    historyCompacted.value = resp.data.compacted;
  } catch (err: any) {
    historyError.value = err.message || t("keys.historyLoadFailed");
  } finally {
    historyLoading.value = false;
  }
};

const loadRevision = async () => {
  if (!clusterId.value || !selectedKey.value) return;
  const revision = Number(revisionInput.value);
  if (!Number.isFinite(revision) || revision <= 0) {
    historyError.value = t("keys.invalidRevision");
    return;
  }
  historyLoading.value = true;
  historyError.value = "";
  try {
    const resp = await api.get<KvItem | null>(`/clusters/${clusterId.value}/kv/item`, {
      params: { key: selectedKey.value.key, revision },
    });
    revisionItem.value = resp.data;
  } catch (err: any) {
    historyError.value = err.message || t("keys.historyLoadFailed");
  } finally {
    historyLoading.value = false;
  }
};
```

- [ ] **Step 4: Add detail panel template**

Add after the key list:

```vue
<aside class="key-detail-panel">
  <div v-if="selectedKey" class="key-detail-content">
    <header class="key-detail-head">
      <div>
        <p class="info-label">{{ t("keys.selectedKey") }}</p>
        <h3 class="mono">{{ selectedKey.key }}</h3>
      </div>
    </header>
    <div class="detail-tabs" role="tablist">
      <button :class="{ active: detailTab === 'details' }" @click="detailTab = 'details'">{{ t("keys.details") }}</button>
      <button :class="{ active: detailTab === 'history' }" @click="detailTab = 'history'; loadHistory()">{{ t("keys.history") }}</button>
      <button :class="{ active: detailTab === 'watch' }" @click="detailTab = 'watch'">{{ t("keys.watch") }}</button>
    </div>
    <section v-if="detailTab === 'details'" class="key-detail-section">
      <dl class="detail-list">
        <dt>{{ t("keys.revision") }}</dt><dd>{{ selectedKey.revision }}</dd>
        <dt>{{ t("keys.version") }}</dt><dd>{{ selectedKey.version }}</dd>
        <dt>{{ t("keys.linkedLease") }}</dt><dd>{{ selectedKey.lease || t("common.notSet") }}</dd>
      </dl>
      <pre class="value-preview mono">{{ selectedKey.value }}</pre>
    </section>
    <section v-if="detailTab === 'history'" class="key-detail-section">
      <div class="revision-query">
        <input v-model="revisionInput" :placeholder="t('keys.revision')" />
        <button class="primary" @click="loadRevision" :disabled="historyLoading">{{ t("common.query") }}</button>
      </div>
      <p v-if="historyError" class="message error">{{ historyError }}</p>
      <p v-if="historyCompacted" class="message warn">{{ t("keys.historyCompacted") }}</p>
      <article v-if="revisionItem" class="history-item">
        <strong>{{ t("keys.revision") }} {{ revisionItem.revision }}</strong>
        <pre class="value-preview mono">{{ revisionItem.value }}</pre>
      </article>
      <article v-for="item in history" :key="item.revision" class="history-item">
        <strong>{{ t("keys.revision") }} {{ item.revision }}</strong>
        <span>{{ t("keys.version") }} {{ item.version }}</span>
        <pre class="value-preview mono">{{ item.value }}</pre>
      </article>
    </section>
    <section v-if="detailTab === 'watch'" class="key-detail-section">
      <p class="hint">{{ t("keys.watchPending") }}</p>
    </section>
  </div>
  <div v-else class="detail-placeholder">{{ t("keys.selectKeyHint") }}</div>
</aside>
```

- [ ] **Step 5: Add CSS**

Add scoped styles:

```css
.key-detail-panel {
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: rgba(0, 0, 0, 0.18);
  padding: 16px;
}

.key-detail-content {
  display: grid;
  gap: 14px;
}

.key-detail-head h3 {
  margin: 4px 0 0;
  font-size: 15px;
  word-break: break-all;
}

.detail-tabs {
  display: inline-flex;
  gap: 4px;
  padding: 4px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  width: fit-content;
}

.detail-tabs button {
  min-height: 32px;
  padding: 5px 12px;
  background: transparent;
  color: var(--muted);
}

.detail-tabs button.active {
  color: var(--primary);
  background: var(--primary-dim);
}

.key-detail-section {
  display: grid;
  gap: 12px;
}

.revision-query {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.value-preview {
  max-height: 240px;
  overflow: auto;
  margin: 0;
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--code-text);
  background: rgba(0, 0, 0, 0.24);
  white-space: pre-wrap;
  word-break: break-word;
}

.history-item {
  display: grid;
  gap: 8px;
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
}
```

- [ ] **Step 6: Add i18n keys**

Under `keys`, add English:

```ts
selectedKey: "Selected key",
details: "Details",
history: "History",
watch: "Watch",
selectKeyHint: "Select a key to view details",
historyLoadFailed: "Failed to load revision history",
historyCompacted: "Older revisions have been compacted by etcd",
invalidRevision: "Enter a valid revision",
watchPending: "Watch will be available after the backend stream is added.",
```

Add Chinese equivalents:

```ts
selectedKey: "已选择 Key",
details: "详情",
history: "历史",
watch: "监听",
selectKeyHint: "选择一个 Key 查看详情",
historyLoadFailed: "加载历史 revision 失败",
historyCompacted: "更早的 revision 已被 etcd 压缩",
invalidRevision: "请输入有效 revision",
watchPending: "后端监听流接入后可用。",
```

- [ ] **Step 7: Verify frontend build**

Run:

```powershell
npm run build
```

Expected: Vite build succeeds.

- [ ] **Step 8: Commit key history UI**

```powershell
git add frontend/src/views/KeysView.vue frontend/src/i18n/locales/en-US.ts frontend/src/i18n/locales/zh-CN.ts
git commit -m "feat: add key revision detail view"
```

## Task 5: Backend SSE Watch API

**Files:**
- Modify: `backend/Cargo.toml`
- Modify: `backend/src/models.rs`
- Modify: `backend/src/etcd.rs`
- Modify: `backend/src/handlers.rs`

- [ ] **Step 1: Add stream dependency**

In `backend/Cargo.toml`, add:

```toml
tokio-stream = "0.1"
```

- [ ] **Step 2: Add watch event model**

In `backend/src/models.rs`, add:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvWatchEvent {
    pub event_type: String,
    pub key: String,
    pub value: Option<String>,
    pub revision: i64,
    pub lease: Option<i64>,
}
```

- [ ] **Step 3: Implement watch channel**

In `backend/src/etcd.rs`, update imports:

```rust
use etcd_client::{Client, DeleteOptions, EventType, GetOptions, LeaseTimeToLiveOptions, PutOptions, WatchOptions};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
```

Include `KvWatchEvent` in model imports.

Add:

```rust
pub async fn watch_prefix(
    cluster: &ResolvedCluster,
    prefix: String,
) -> Result<ReceiverStream<Result<KvWatchEvent>>> {
    let prefix = prefix.trim().to_string();
    if prefix.is_empty() {
        return Err(AppError::Validation("prefix is required".to_string()));
    }

    let endpoints = normalize_endpoints(&cluster.endpoints);
    if endpoints.is_empty() {
        return Err(AppError::Validation("cluster has no endpoint".to_string()));
    }

    let client = connect_client(&endpoints).await?;
    let mut watch_client = client.watch_client();
    let options = Some(WatchOptions::new().with_prefix());
    let (_watcher, mut stream) = watch_client
        .watch(prefix, options)
        .await
        .map_err(|err| AppError::Internal(format!("etcd watch failed: {err}")))?;

    let (tx, rx) = mpsc::channel(100);
    tokio::spawn(async move {
        while let Ok(Some(resp)) = stream.message().await {
            for event in resp.events() {
                let Some(kv) = event.kv() else { continue };
                let event_type = match event.event_type() {
                    EventType::Put => "put",
                    EventType::Delete => "delete",
                };
                let item = KvWatchEvent {
                    event_type: event_type.to_string(),
                    key: String::from_utf8_lossy(kv.key()).to_string(),
                    value: if event.event_type() == EventType::Put {
                        Some(bytes_to_string(kv.value()))
                    } else {
                        None
                    },
                    revision: kv.mod_revision(),
                    lease: if kv.lease() == 0 { None } else { Some(kv.lease()) },
                };
                if tx.send(Ok(item)).await.is_err() {
                    return;
                }
            }
        }
    });

    Ok(ReceiverStream::new(rx))
}
```

- [ ] **Step 4: Add SSE handler**

In `backend/src/handlers.rs`, update imports:

```rust
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{
        sse::{Event, Sse},
        IntoResponse,
    },
    routing::{get, post, put},
    Json, Router,
};
use std::convert::Infallible;
use tokio_stream::StreamExt;
```

Add route:

```rust
.route("/api/clusters/:id/kv/watch", get(watch_kv))
```

Add query:

```rust
#[derive(Deserialize)]
pub struct KvWatchQuery {
    pub prefix: String,
}
```

Add handler:

```rust
async fn watch_kv(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<KvWatchQuery>,
) -> Result<Sse<impl tokio_stream::Stream<Item = std::result::Result<Event, Infallible>>>> {
    let user = auth::require_user(&headers, &state.config)?;
    require_permission(&user, "key:read")?;
    let cluster = etcd::resolve_cluster_by_id(&state, &id).await?;
    let stream = etcd::watch_prefix(&cluster, query.prefix).await?;
    let events = stream.map(|item| {
        let event = match item {
            Ok(payload) => Event::default()
                .event(payload.event_type.clone())
                .json_data(payload)
                .unwrap_or_else(|_| Event::default().event("error").data("serialize failed")),
            Err(err) => Event::default().event("error").data(err.to_string()),
        };
        Ok::<Event, Infallible>(event)
    });
    Ok(Sse::new(events))
}
```

- [ ] **Step 5: Verify backend compiles**

Run:

```powershell
cargo check
```

Expected: dependencies resolve and backend compiles.

- [ ] **Step 6: Commit watch backend**

```powershell
git add backend/Cargo.toml backend/Cargo.lock backend/src/models.rs backend/src/etcd.rs backend/src/handlers.rs
git commit -m "feat: add key watch SSE API"
```

## Task 6: Frontend Watch UI

**Files:**
- Modify: `frontend/src/views/KeysView.vue`
- Modify: `frontend/src/i18n/locales/en-US.ts`
- Modify: `frontend/src/i18n/locales/zh-CN.ts`

- [ ] **Step 1: Add watch state**

In `KeysView.vue`, add:

```ts
interface WatchEventItem {
  event_type: "put" | "delete" | "error";
  key: string;
  value?: string;
  revision: number;
  lease?: number;
}

const watchSource = ref<EventSource | null>(null);
const watchEvents = ref<WatchEventItem[]>([]);
const watchPrefix = ref("");
const watchStatus = ref<"idle" | "running" | "paused" | "error">("idle");
const watchFilter = ref<"all" | "put" | "delete">("all");
const watchPaused = ref(false);
```

- [ ] **Step 2: Add watch controls**

Add:

```ts
const visibleWatchEvents = computed(() => {
  if (watchFilter.value === "all") return watchEvents.value;
  return watchEvents.value.filter((item) => item.event_type === watchFilter.value);
});

const appendWatchEvent = (event: WatchEventItem) => {
  if (watchPaused.value) return;
  watchEvents.value = [event, ...watchEvents.value].slice(0, 500);
};

const startWatch = () => {
  if (!clusterId.value) return;
  stopWatch();
  const prefixToWatch = watchPrefix.value.trim() || selectedKey.value?.key || prefix.value;
  if (!prefixToWatch) return;
  watchStatus.value = "running";
  watchPaused.value = false;
  const params = new URLSearchParams({ prefix: prefixToWatch });
  const source = new EventSource(`/api/clusters/${clusterId.value}/kv/watch?${params.toString()}`);
  source.addEventListener("put", (event) => appendWatchEvent(JSON.parse((event as MessageEvent).data)));
  source.addEventListener("delete", (event) => appendWatchEvent(JSON.parse((event as MessageEvent).data)));
  source.addEventListener("error", () => {
    watchStatus.value = "error";
  });
  watchSource.value = source;
};

const stopWatch = () => {
  watchSource.value?.close();
  watchSource.value = null;
  watchStatus.value = "idle";
  watchPaused.value = false;
};

const toggleWatchPause = () => {
  watchPaused.value = !watchPaused.value;
  watchStatus.value = watchPaused.value ? "paused" : "running";
};

const clearWatchEvents = () => {
  watchEvents.value = [];
};
```

Add to `onUnmounted`:

```ts
stopWatch();
```

If `KeysView.vue` does not currently import `onUnmounted`, update the Vue import.

- [ ] **Step 3: Replace watch pending UI**

Replace the watch section from Task 4 with:

```vue
<section v-if="detailTab === 'watch'" class="key-detail-section">
  <div class="watch-controls">
    <input v-model="watchPrefix" :placeholder="selectedKey?.key || prefix" />
    <button class="primary" @click="startWatch" :disabled="watchStatus === 'running'">{{ t("keys.startWatch") }}</button>
    <button class="ghost" @click="toggleWatchPause" :disabled="watchStatus === 'idle'">{{ watchPaused ? t("keys.resumeWatch") : t("keys.pauseWatch") }}</button>
    <button class="ghost" @click="stopWatch" :disabled="watchStatus === 'idle'">{{ t("keys.stopWatch") }}</button>
    <button class="ghost" @click="clearWatchEvents">{{ t("common.clear") }}</button>
  </div>
  <div class="watch-filters">
    <button :class="{ active: watchFilter === 'all' }" @click="watchFilter = 'all'">{{ t("common.all") }}</button>
    <button :class="{ active: watchFilter === 'put' }" @click="watchFilter = 'put'">PUT</button>
    <button :class="{ active: watchFilter === 'delete' }" @click="watchFilter = 'delete'">DELETE</button>
  </div>
  <p class="hint">{{ t("keys.watchStatus") }}: {{ watchStatus }}</p>
  <article v-for="event in visibleWatchEvents" :key="`${event.revision}-${event.key}-${event.event_type}`" class="watch-event">
    <header>
      <strong>{{ event.event_type.toUpperCase() }}</strong>
      <span>{{ t("keys.revision") }} {{ event.revision }}</span>
    </header>
    <p class="mono">{{ event.key }}</p>
    <pre v-if="event.value" class="value-preview mono">{{ event.value }}</pre>
  </article>
</section>
```

- [ ] **Step 4: Add CSS**

```css
.watch-controls,
.watch-filters {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.watch-controls input {
  min-width: min(280px, 100%);
  flex: 1;
}

.watch-filters button.active {
  color: var(--primary);
  background: var(--primary-dim);
}

.watch-event {
  display: grid;
  gap: 8px;
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: rgba(0, 0, 0, 0.18);
}

.watch-event header {
  display: flex;
  justify-content: space-between;
  gap: 10px;
  color: var(--muted);
  font-size: 12px;
}

.watch-event p {
  margin: 0;
  word-break: break-all;
}
```

- [ ] **Step 5: Add i18n keys**

English:

```ts
startWatch: "Start watch",
pauseWatch: "Pause",
resumeWatch: "Resume",
stopWatch: "Stop",
watchStatus: "Watch status",
```

Chinese:

```ts
startWatch: "开始监听",
pauseWatch: "暂停",
resumeWatch: "继续",
stopWatch: "停止",
watchStatus: "监听状态",
```

- [ ] **Step 6: Verify frontend build**

Run:

```powershell
npm run build
```

Expected: Vite build succeeds.

- [ ] **Step 7: Commit watch UI**

```powershell
git add frontend/src/views/KeysView.vue frontend/src/i18n/locales/en-US.ts frontend/src/i18n/locales/zh-CN.ts
git commit -m "feat: add key watch UI"
```

## Task 7: Final Verification And Docs

**Files:**
- Modify: `README.en.md`
- Modify: `README.zh-CN.md`

- [ ] **Step 1: Run backend check**

```powershell
cargo check
```

Expected: backend compiles.

- [ ] **Step 2: Run frontend build**

```powershell
npm run build
```

Expected: frontend compiles. Existing chunk-size warning is acceptable.

- [ ] **Step 3: Update README feature bullets**

In both README files, add bullets for:

```markdown
- Inspect endpoint-level etcd health and raft status.
- Read key values at a specific revision when etcd still retains the revision.
- Watch key or prefix changes in real time.
```

Use Chinese equivalents in `README.zh-CN.md`.

- [ ] **Step 4: Document compacted revision behavior**

Add under configuration or usage notes:

```markdown
Revision history depends on etcd MVCC retention. Revisions compacted by etcd cannot be read from EtcdPilot.
```

Use Chinese equivalent in `README.zh-CN.md`.

- [ ] **Step 5: Final status check**

Run:

```powershell
git status --short
```

Expected: only intended files are modified.

- [ ] **Step 6: Commit docs**

```powershell
git add README.en.md README.zh-CN.md
git commit -m "docs: describe observability features"
```

## Self-Review

- Spec coverage: endpoint status is covered by Tasks 1-2; revision read/history by Tasks 3-4; SSE watch by Tasks 5-6; docs/final verification by Task 7.
- Placeholder scan: the plan contains concrete files, commands, and code snippets for each task.
- Scope: snapshot restore, defrag, alarm disarm, and member removal remain out of scope.
- Type consistency: backend DTO names are `EndpointStatusItem`, `EndpointStatusResponse`, `KvHistoryResponse`, and `KvWatchEvent`; frontend mirrors these names where needed.
