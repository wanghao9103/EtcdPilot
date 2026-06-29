use std::time::Duration;

use etcd_client::{
    Client, DeleteOptions, EventType, GetOptions, LeaseTimeToLiveOptions, PutOptions,
    WatchOptions,
};
use tokio::sync::mpsc;
use tokio::time::timeout;
use tokio_stream::wrappers::ReceiverStream;

use crate::{
    db,
    error::{AppError, Result},
    models::{EndpointStatusItem, EndpointStatusResponse, KvHistoryResponse, KvItem, KvWatchEvent},
    AppState,
};

pub struct ResolvedCluster {
    pub cluster_id: String,
    pub name: String,
    pub endpoints: Vec<String>,
    pub readonly: bool,
    pub source: String,
}

pub async fn resolve_cluster_by_id(state: &AppState, id: &str) -> Result<ResolvedCluster> {
    for config_cluster in &state.config.clusters {
        if config_cluster.id == id {
            return Ok(ResolvedCluster {
                cluster_id: config_cluster.id.clone(),
                name: config_cluster.name.clone(),
                endpoints: config_cluster.endpoints.clone(),
                readonly: config_cluster.readonly,
                source: "config".to_string(),
            });
        }
    }

    let rows = db::list_db_clusters(&state.pool).await?;
    for row in rows {
        if row.id == id {
            if row.disabled {
                return Err(AppError::Validation("cluster is disabled".to_string()));
            }
            return Ok(ResolvedCluster {
                cluster_id: row.id,
                name: row.name,
                endpoints: row.endpoints,
                readonly: row.readonly,
                source: "database".to_string(),
            });
        }
    }

    Err(AppError::NotFound("cluster not found".to_string()))
}

pub async fn ping_cluster(cluster: &ResolvedCluster) -> Result<bool> {
    let endpoints = normalize_endpoints(&cluster.endpoints);
    if endpoints.is_empty() {
        return Err(AppError::Validation("cluster has no endpoint".to_string()));
    }

    connect_client(&endpoints)
        .await
        .map(|_| true)
        .map_err(|err| AppError::Internal(format!("cluster is unavailable: {err}")))
}

pub async fn status(cluster: &ResolvedCluster) -> Result<serde_json::Value> {
    let endpoints = normalize_endpoints(&cluster.endpoints);
    if endpoints.is_empty() {
        return Err(AppError::Validation("cluster has no endpoint".to_string()));
    }

    let client = connect_client(&endpoints).await?;
    let mut maintenance = client.maintenance_client();
    let response = timeout(Duration::from_secs(3), maintenance.status())
        .await
        .map_err(|_| AppError::Validation("etcd status timeout".to_string()))?
        .map_err(|err| AppError::Internal(format!("etcd status failed: {err}")))?;

    Ok(serde_json::json!({
        "cluster_id": cluster.cluster_id,
        "name": cluster.name,
        "source": cluster.source,
        "status": "ok",
        "endpoint": endpoints.first().cloned().unwrap_or_default(),
        "leader": response.leader(),
        "version": response.version(),
        "raft_index": response.raft_index(),
        "raft_term": response.raft_term(),
        "raft_applied_index": response.raft_applied_index(),
        "raft_used_db_size": response.raft_used_db_size(),
        "db_size": response.db_size(),
        "errors": response.errors(),
        "is_learner": response.is_learner(),
    }))
}

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
            is_learner: None,
            error: Some(err.to_string()),
        }));
    }

    Ok(EndpointStatusResponse {
        cluster_id: cluster.cluster_id.clone(),
        endpoints: items,
    })
}

pub async fn members(cluster: &ResolvedCluster) -> Result<Vec<serde_json::Value>> {
    let endpoints = normalize_endpoints(&cluster.endpoints);
    if endpoints.is_empty() {
        return Err(AppError::Validation("cluster has no endpoint".to_string()));
    }

    let client = connect_client(&endpoints).await?;
    let mut cluster_client = client.cluster_client();
    let response = timeout(Duration::from_secs(3), cluster_client.member_list())
        .await
        .map_err(|_| AppError::Validation("etcd member list timeout".to_string()))?
        .map_err(|err| AppError::Internal(format!("etcd member list failed: {err}")))?;

    Ok(response
        .members()
        .iter()
        .map(|member| {
            serde_json::json!({
                "id": member.id(),
                "name": member.name(),
                "is_learner": member.is_learner(),
                "peer_urls": member.peer_urls(),
                "client_urls": member.client_urls(),
            })
        })
        .collect())
}

pub async fn list_kv(cluster: &ResolvedCluster, prefix: Option<String>) -> Result<Vec<KvItem>> {
    let prefix = prefix.unwrap_or_default();
    let key = if prefix.is_empty() {
        "/".to_string()
    } else {
        prefix
    };
    let options = GetOptions::new().with_prefix();

    let mut kvs = fetch_kv(cluster, key, Some(options)).await?;
    kvs.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(kvs)
}

pub async fn list_leases(cluster: &ResolvedCluster) -> Result<Vec<serde_json::Value>> {
    let endpoints = normalize_endpoints(&cluster.endpoints);
    if endpoints.is_empty() {
        return Err(AppError::Validation("cluster has no endpoint".to_string()));
    }

    let client = connect_client(&endpoints).await?;
    let mut lease_client = client.lease_client();
    let response = timeout(Duration::from_secs(5), lease_client.leases())
        .await
        .map_err(|_| AppError::Validation("lease list timeout".to_string()))?
        .map_err(|err| AppError::Internal(format!("etcd lease list failed: {err}")))?;

    let mut leases = Vec::with_capacity(response.leases().len());
    for status in response.leases() {
        let lease_id = status.id();
        let details = timeout(
            Duration::from_secs(5),
            lease_client.time_to_live(lease_id, Some(LeaseTimeToLiveOptions::new().with_keys())),
        )
        .await
        .map_err(|_| AppError::Validation("lease ttl timeout".to_string()))?
        .map_err(|err| AppError::Internal(format!("etcd lease ttl failed: {err}")))?;

        let keys: Vec<String> = details
            .keys()
            .iter()
            .map(|key| String::from_utf8_lossy(key).to_string())
            .collect();

        leases.push(serde_json::json!({
            "id": lease_id,
            "ttl": details.ttl(),
            "granted_ttl": details.granted_ttl(),
            "keys": keys,
            "keys_count": keys.len(),
        }));
    }

    Ok(leases)
}

pub async fn get_lease(cluster: &ResolvedCluster, lease_id: i64) -> Result<serde_json::Value> {
    let endpoints = normalize_endpoints(&cluster.endpoints);
    if endpoints.is_empty() {
        return Err(AppError::Validation("cluster has no endpoint".to_string()));
    }

    let client = connect_client(&endpoints).await?;
    let mut lease_client = client.lease_client();
    let details = timeout(
        Duration::from_secs(5),
        lease_client.time_to_live(lease_id, Some(LeaseTimeToLiveOptions::new().with_keys())),
    )
    .await
    .map_err(|_| AppError::Validation("lease ttl timeout".to_string()))?
    .map_err(|err| AppError::Internal(format!("etcd lease ttl failed: {err}")))?;

    let keys: Vec<String> = details
        .keys()
        .iter()
        .map(|key| String::from_utf8_lossy(key).to_string())
        .collect();

    Ok(serde_json::json!({
        "id": lease_id,
        "ttl": details.ttl(),
        "granted_ttl": details.granted_ttl(),
        "keys": keys,
        "keys_count": keys.len(),
    }))
}

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
                    lease: if kv.lease() == 0 {
                        None
                    } else {
                        Some(kv.lease())
                    },
                };
                if tx.send(Ok(item)).await.is_err() {
                    return;
                }
            }
        }
    });

    Ok(ReceiverStream::new(rx))
}

pub async fn put_kv(
    cluster: &ResolvedCluster,
    key: String,
    value: String,
    lease: Option<i64>,
) -> Result<()> {
    if cluster.readonly {
        return Err(AppError::Validation("cluster is read-only".to_string()));
    }
    if key.trim().is_empty() {
        return Err(AppError::Validation("key is required".to_string()));
    }

    let endpoints = normalize_endpoints(&cluster.endpoints);
    if endpoints.is_empty() {
        return Err(AppError::Validation("cluster has no endpoint".to_string()));
    }

    let client = connect_client(&endpoints).await?;
    let mut kv_client = client.kv_client();
    let put_opts = if let Some(lease) = lease {
        PutOptions::new().with_lease(lease)
    } else {
        PutOptions::new()
    };
    kv_client
        .put(key, value, Some(put_opts))
        .await
        .map_err(|err| AppError::Internal(format!("etcd put failed: {err}")))?;
    Ok(())
}

pub async fn delete_kv(cluster: &ResolvedCluster, key: String) -> Result<()> {
    if cluster.readonly {
        return Err(AppError::Validation("cluster is read-only".to_string()));
    }
    let key = key.trim().to_string();
    if key.is_empty() {
        return Err(AppError::Validation("key is required".to_string()));
    }

    let endpoints = normalize_endpoints(&cluster.endpoints);
    if endpoints.is_empty() {
        return Err(AppError::Validation("cluster has no endpoint".to_string()));
    }

    let client = connect_client(&endpoints).await?;
    let mut kv_client = client.kv_client();
    kv_client
        .delete(key, Some(DeleteOptions::new()))
        .await
        .map_err(|err| AppError::Internal(format!("etcd delete failed: {err}")))?;
    Ok(())
}

fn normalize_endpoints(raw: &[String]) -> Vec<String> {
    raw.iter()
        .filter_map(|raw_ep| {
            let endpoint = raw_ep.trim();
            if endpoint.is_empty() {
                return None;
            }
            if endpoint.contains("://") {
                Some(endpoint.to_string())
            } else if endpoint.contains(':') {
                Some(format!("http://{endpoint}"))
            } else {
                Some(format!("http://{endpoint}:2379"))
            }
        })
        .collect()
}

fn parse_kv_to_item(key: &str, value: &[u8], kv: &etcd_client::KeyValue) -> KvItem {
    KvItem {
        key: key.to_string(),
        value: bytes_to_string(value),
        revision: kv.mod_revision(),
        version: kv.version(),
        create_revision: kv.create_revision(),
        mod_revision: kv.mod_revision(),
        lease: if kv.lease() == 0 {
            None
        } else {
            Some(kv.lease())
        },
    }
}

fn bytes_to_string(data: &[u8]) -> String {
    String::from_utf8_lossy(data).to_string()
}

async fn connect_client(endpoints: &[String]) -> Result<Client> {
    let connected = timeout(
        Duration::from_secs(3),
        Client::connect(endpoints.to_vec(), None),
    )
    .await
    .map_err(|_| AppError::Validation("connect timeout".to_string()))?;
    connected.map_err(|err| AppError::Internal(format!("etcd connect failed: {err}")))
}

async fn fetch_kv(
    cluster: &ResolvedCluster,
    key: String,
    options: Option<GetOptions>,
) -> Result<Vec<KvItem>> {
    let endpoints = normalize_endpoints(&cluster.endpoints);
    if endpoints.is_empty() {
        return Err(AppError::Validation("cluster has no endpoint".to_string()));
    }

    let client = connect_client(&endpoints).await?;
    let mut kv_client = client.kv_client();
    let is_single = options.is_none();
    let response = timeout(Duration::from_secs(5), kv_client.get(key, options))
        .await
        .map_err(|_| AppError::Validation("kv get timeout".to_string()))?
        .map_err(|err| AppError::Internal(format!("etcd get failed: {err}")))?;

    let mut kvs: Vec<KvItem> = response
        .kvs()
        .iter()
        .map(|kv| {
            parse_kv_to_item(
                &String::from_utf8_lossy(kv.key()).to_string(),
                kv.value(),
                kv,
            )
        })
        .collect();

    if is_single && kvs.len() > 1 {
        kvs.truncate(1);
    }

    Ok(kvs)
}
