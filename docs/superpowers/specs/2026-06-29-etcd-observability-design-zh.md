# EtcdPilot etcd 诊断与实时观测增强设计

## 背景

EtcdPilot 当前已经具备集群管理、Key/Value 浏览编辑、服务注册拓扑、租约和审计能力。作为 etcd 管理界面，下一阶段应补齐 etcd 运维中最常用的观测能力：实时监听 key 变化、按 revision 查看历史值、查看更完整的集群健康状态。

本设计聚焦第一期可交付能力，避免引入 snapshot restore、member remove、defrag 等高风险操作。

## 目标

第一期完成后，用户可以：

- 查看每个 etcd endpoint 的健康状态和关键指标。
- 查看 member、leader、raft、db size 等集群状态信息。
- 点击 key 后查看当前 key 详情。
- 按 revision 读取 key 的历史值。
- 对指定 key 或 prefix 开启实时 watch，看到 put/delete 事件流。

## 非目标

第一期不做：

- snapshot 备份与恢复。
- defrag、alarm disarm、member remove 等危险运维操作。
- 自动回滚历史值。
- 长期审计归档。etcd revision 历史受 compact 影响，不作为完整审计来源。

## 总体方案

保留现有页面结构，在 `KeysView` 和 `ClustersView` 内增强能力：

- `KeysView` 增加 key 详情区域，提供 Details、History、Watch 三类信息。
- `ClustersView` 增强现有状态和成员区域，展示 endpoint 级别状态和 raft 指标。
- 后端在现有 `etcd.rs` 封装中增加 revision 查询、endpoint status、watch stream 能力。
- Watch 第一版使用 SSE，前端通过浏览器原生 `EventSource` 接收事件。

## 后端 API

新增或增强接口：

```text
GET /api/clusters/:id/endpoints/status
GET /api/clusters/:id/kv/item?key=:key&revision=:revision
GET /api/clusters/:id/kv/history?key=:key&limit=:limit
GET /api/clusters/:id/kv/watch?prefix=:prefix
```

### Endpoint Status

`GET /api/clusters/:id/endpoints/status` 返回每个 endpoint 的探测结果。

响应示例：

```json
{
  "cluster_id": "demo",
  "endpoints": [
    {
      "endpoint": "http://127.0.0.1:2379",
      "reachable": true,
      "version": "3.5.11",
      "leader": 123,
      "raft_term": 8,
      "raft_index": 1024,
      "raft_applied_index": 1024,
      "db_size": 1048576,
      "error": ""
    }
  ]
}
```

### Revision Read

现有 `GET /api/clusters/:id/kv/item` 支持可选 `revision` 参数。未传 revision 时保持当前行为，传入 revision 时通过 etcd `GetOptions::with_revision` 查询历史值。

如果 revision 已被 compact，返回结构化错误：

```json
{
  "code": "REVISION_COMPACTED",
  "message": "revision has been compacted"
}
```

### History

`GET /api/clusters/:id/kv/history` 第一版不承诺完整历史列表。实现策略为：

- 输入 key 和 limit。
- 从当前 key 的 `mod_revision` 开始向前尝试读取若干 revision。
- 去重相同值，返回最多 `limit` 条可读取记录。
- 遇到 compacted revision 时停止并返回 `compacted: true`。

这是一种轻量历史视图，适合第一期；后续如果需要完整变更记录，应通过 watch 持久化到本地审计表。

### Watch

`GET /api/clusters/:id/kv/watch?prefix=/services/` 使用 SSE：

```text
Content-Type: text/event-stream
```

事件示例：

```json
{
  "type": "put",
  "key": "/services/demo/app",
  "revision": 12345,
  "value": "{...}",
  "lease": 123
}
```

服务端要求：

- 连接断开时停止 etcd watch。
- 后端对 watch prefix 做非空校验。
- 每个 watch 连接只监听一个 key 或 prefix。
- 对 value 做 UTF-8 安全处理，非 UTF-8 value 返回 base64 标记。

## 前端设计

### KeysView

在现有 key 列表旁增加 key 详情区。窄屏时详情区放到列表下方。

详情区包含三个 tab：

- Details：当前 key、value、revision、version、lease、create revision、mod revision。
- History：输入或选择 revision，展示历史值和当前值差异。
- Watch：监听当前 key 或当前 prefix。

Watch 区域能力：

- 开始监听。
- 暂停/继续。
- 停止监听。
- 清空事件。
- 事件类型过滤：全部、PUT、DELETE。
- 最近最多保留 500 条事件，避免页面卡顿。

事件行点击后展示完整 value。长 value 默认折叠。

### ClustersView

增强现有集群卡片：

- 状态摘要：reachable endpoints、leader、db size、raft index。
- Endpoint 明细：每个 endpoint 的 reachable、version、leader、raft term/index、db size、错误信息。
- Member 明细：member id、name、peer URLs、client URLs、learner 状态。

现有“加载状态”和“查看节点”按钮可以保留，但状态展示改为结构化区域，而不是原始 JSON 为主。原始 JSON 可收进技术详情。

## 权限

沿用当前权限模型：

- `cluster:read`：查看 endpoint status、members。
- `kv:read`：读取 key、revision、watch。
- `kv:write`：第一期不新增写操作，后续回滚历史值时再使用。
- admin：未来危险操作使用。

## 错误处理

需要明确处理：

- endpoint 不可达。
- watch 连接断开。
- revision 已 compact。
- key 不存在。
- value 过大。
- value 非 UTF-8。
- 权限不足。
- prefix watch 事件过多。

前端对 watch 断开显示状态，不自动无限重连。用户可以手动重新开始监听。

## 数据和状态

第一期不新增数据库表。

原因：

- Endpoint status 和 revision read 都是即时查询。
- Watch 第一版只做实时查看，不做持久化。
- 完整变更审计属于后续独立能力。

## 测试策略

后端：

- `kv/item` 不传 revision 时保持兼容。
- `kv/item` 传 revision 时能读取指定 revision。
- compacted revision 返回结构化错误。
- endpoint status 对单 endpoint 失败不会导致整组失败。
- watch 连接关闭后释放资源。

前端：

- Key 详情区在未选择 key 时显示空状态。
- History compacted、not found、正常返回三种状态可见。
- Watch 可开始、暂停、停止、清空。
- Watch 事件超过 500 条时保留最近事件。
- ClustersView 能展示部分 endpoint 失败。

## 实施顺序

1. 后端实现 endpoint status 聚合。
2. 前端增强 ClustersView 状态展示。
3. 后端支持按 revision 读取 key。
4. 前端增加 Key 详情和 History tab。
5. 后端实现 SSE watch。
6. 前端增加 Watch tab。

## 风险

- etcd 历史 revision 受 compact 影响，不能承诺完整历史。
- Prefix watch 在高频写入场景可能产生大量事件，需要前端限制保留数量。
- SSE 在代理环境中可能受缓冲影响，部署文档后续需要补充反向代理配置建议。
- 非 UTF-8 value 不能直接按文本展示，需要明确标记并提供 base64 展示。

## 验收标准

- 用户可以在集群页面看到每个 endpoint 的健康和 raft 指标。
- 用户可以在 Key 页面查看当前 key 详情。
- 用户可以输入 revision 查看历史值。
- compacted revision 有清晰提示。
- 用户可以监听 key 或 prefix，并看到实时 put/delete 事件。
- Watch 断开、停止、暂停状态在 UI 中明确可见。
