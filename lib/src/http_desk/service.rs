//! 集合/环境/请求�?CRUD——全部通过 `Arc<dyn FileOps>` 读写已打开工作区目录下�?//! `.rock_desk/http/` 子目录（docs/HTTP_DESKTOP_PLAN.md §4.4）。本�?SSH/Agent
//! 三种工作区走的是同一套代码，不在这里区分——差异已经被 `FileOps` trait 吸收掉�?//!
//! 没有独立的集�?环境 SQLite 表：集合列表靠扫 `collections/` 目录发现，环境列�?//! 靠扫某个集合�?`environments/` 目录发现，YAML 文件本身就是唯一真相�?
use uuid::Uuid;

use crate::error::AppError;
use roc_desk_common::fsops::FileOps;

use super::model::*;

pub fn http_root(workspace_root: &str) -> String {
    format!("{}/.rock_desk/http", workspace_root.trim_end_matches('/'))
}

fn collections_dir(workspace_root: &str) -> String {
    format!("{}/collections", http_root(workspace_root))
}

fn collection_dir(workspace_root: &str, slug: &str) -> String {
    format!("{}/{}", collections_dir(workspace_root), slug)
}

fn collection_meta_path(workspace_root: &str, slug: &str) -> String {
    format!("{}/collection.yaml", collection_dir(workspace_root, slug))
}

fn requests_dir(workspace_root: &str, slug: &str) -> String {
    format!("{}/requests", collection_dir(workspace_root, slug))
}

fn environments_dir(workspace_root: &str, slug: &str) -> String {
    format!("{}/environments", collection_dir(workspace_root, slug))
}

fn environment_path(workspace_root: &str, slug: &str, env_id: &str) -> String {
    format!("{}/{}.yaml", environments_dir(workspace_root, slug), env_id)
}

fn global_vars_path(workspace_root: &str) -> String {
    format!("{}/global.yaml", http_root(workspace_root))
}

/// 把集�?请求名转成路径安全的 slug——中日韩�?Unicode 字母数字原样保留（很�?/// 用户会用中文集合名，比如"用户模块"），只把其它字符（空格、标点）折成 `-`�?pub fn slugify(name: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in name.trim().chars() {
        if ch.is_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
    }
    let trimmed = out.trim_end_matches('-').to_string();
    if trimmed.is_empty() {
        Uuid::new_v4().to_string()
    } else {
        trimmed
    }
}

/// 列目录时�?目录还不存在"（首次打开、还没建过任何集合）当成空列表，而不是报�?/// 一路冒泡到前端�?toast——三�?`FileOps` 实现�?不存�?的错误分类不完全一�?/// （本地是 `NotFound`，Agent 侧可能是 `Internal`），这里索性对 `list_dir` 失败
/// 统一降级成空列表：代价是短暂掩盖�?目录其实因为别的原因读不出来"这种更少见的
/// 错误，但换来跨三种工作区类型一致的首次打开体验，换回来的价值更大；真出问题�?/// 用户创建集合/请求那一步的写入会正常报错，不会被这里悄悄吞掉�?async fn list_dir_or_empty(
    file_ops: &dyn FileOps,
    path: &str,
) -> Vec<roc_desk_common::fsops::FileEntry> {
    file_ops.list_dir(path).await.unwrap_or_default()
}

async fn read_yaml<T: serde::de::DeserializeOwned>(
    file_ops: &dyn FileOps,
    path: &str,
) -> Result<T, AppError> {
    let content = file_ops.read_file(path).await?;
    serde_yaml::from_str(&content.text)
        .map_err(|e| AppError::Internal(format!("解析 {path} 失败：{e}")))
}

async fn write_yaml<T: serde::Serialize>(
    file_ops: &dyn FileOps,
    path: &str,
    value: &T,
) -> Result<(), AppError> {
    let text = serde_yaml::to_string(value)
        .map_err(|e| AppError::Internal(format!("序列�?{path} 失败：{e}")))?;
    file_ops.write_file(path, &text, None).await?;
    Ok(())
}

// ---------------------------------------------------------------------
// 集合
// ---------------------------------------------------------------------

pub async fn list_collections(
    file_ops: &dyn FileOps,
    workspace_root: &str,
) -> Result<Vec<HttpCollectionSummary>, AppError> {
    let dir = collections_dir(workspace_root);
    let entries = list_dir_or_empty(file_ops, &dir).await;
    let mut out = Vec::new();
    for entry in entries.into_iter().filter(|e| e.is_dir) {
        let slug = entry.name;
        let meta_path = collection_meta_path(workspace_root, &slug);
        let meta: HttpCollectionMeta = match read_yaml(file_ops, &meta_path).await {
            Ok(m) => m,
            Err(_) => continue, // 目录里没有合�?collection.yaml，跳过（不是一个集合）
        };
        let request_count = list_requests(file_ops, workspace_root, &slug)
            .await
            .map(|r| r.len())
            .unwrap_or(0);
        out.push(HttpCollectionSummary {
            slug,
            name: meta.name,
            description: meta.description,
            request_count,
        });
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(out)
}

pub async fn create_collection(
    file_ops: &dyn FileOps,
    workspace_root: &str,
    name: &str,
) -> Result<HttpCollectionSummary, AppError> {
    let base_slug = slugify(name);
    let existing = list_collections(file_ops, workspace_root).await?;
    let mut slug = base_slug.clone();
    let mut n = 2;
    while existing.iter().any(|c| c.slug == slug) {
        slug = format!("{base_slug}-{n}");
        n += 1;
    }
    file_ops.create_dir(&requests_dir(workspace_root, &slug)).await?;
    file_ops
        .create_dir(&environments_dir(workspace_root, &slug))
        .await?;
    let meta = HttpCollectionMeta {
        name: name.to_string(),
        description: None,
        auth: AuthConfig::None,
        variables: Vec::new(),
    };
    write_yaml(file_ops, &collection_meta_path(workspace_root, &slug), &meta).await?;
    // 新集合默认给一�?默认环境"，避免用户第一次进来就得先手动新建环境才能填变量�?    let default_env = EnvironmentDef::new(Uuid::new_v4().to_string(), "默认环境".to_string());
    write_yaml(
        file_ops,
        &environment_path(workspace_root, &slug, &default_env.id),
        &default_env,
    )
    .await?;
    Ok(HttpCollectionSummary {
        slug,
        name: meta.name,
        description: meta.description,
        request_count: 0,
    })
}

pub async fn delete_collection(
    file_ops: &dyn FileOps,
    workspace_root: &str,
    slug: &str,
) -> Result<(), AppError> {
    file_ops
        .delete(&collection_dir(workspace_root, slug), true)
        .await
}

pub async fn rename_collection(
    file_ops: &dyn FileOps,
    workspace_root: &str,
    slug: &str,
    new_name: &str,
) -> Result<(), AppError> {
    let path = collection_meta_path(workspace_root, slug);
    let mut meta: HttpCollectionMeta = read_yaml(file_ops, &path).await?;
    meta.name = new_name.to_string();
    write_yaml(file_ops, &path, &meta).await
}

pub async fn get_collection_meta(
    file_ops: &dyn FileOps,
    workspace_root: &str,
    slug: &str,
) -> Result<HttpCollectionMeta, AppError> {
    read_yaml(file_ops, &collection_meta_path(workspace_root, slug)).await
}

/// 保存整份集合元数据（含集合级 Auth/变量，见 §3.3 Auth Tab �?集合"这一档�?/// §4.5 �?4 层集合变量）——`rename_collection` 是这个函数针�?只改名字"场景�?/// 一个便捷封装，两者都写同一个文件�?pub async fn save_collection_meta(
    file_ops: &dyn FileOps,
    workspace_root: &str,
    slug: &str,
    meta: &HttpCollectionMeta,
) -> Result<(), AppError> {
    write_yaml(file_ops, &collection_meta_path(workspace_root, slug), meta).await
}

// ---------------------------------------------------------------------
// 环境
// ---------------------------------------------------------------------

pub async fn list_environments(
    file_ops: &dyn FileOps,
    workspace_root: &str,
    slug: &str,
) -> Result<Vec<EnvironmentDef>, AppError> {
    let dir = environments_dir(workspace_root, slug);
    let entries = list_dir_or_empty(file_ops, &dir).await;
    let mut out = Vec::new();
    for entry in entries.into_iter().filter(|e| !e.is_dir && e.name.ends_with(".yaml")) {
        let path = format!("{dir}/{}", entry.name);
        if let Ok(env) = read_yaml::<EnvironmentDef>(file_ops, &path).await {
            out.push(env);
        }
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(out)
}

pub async fn save_environment(
    file_ops: &dyn FileOps,
    workspace_root: &str,
    slug: &str,
    mut env: EnvironmentDef,
) -> Result<EnvironmentDef, AppError> {
    if env.id.is_empty() {
        env.id = Uuid::new_v4().to_string();
    }
    write_yaml(
        file_ops,
        &environment_path(workspace_root, slug, &env.id),
        &env,
    )
    .await?;
    Ok(env)
}

pub async fn delete_environment(
    file_ops: &dyn FileOps,
    workspace_root: &str,
    slug: &str,
    env_id: &str,
) -> Result<(), AppError> {
    file_ops
        .delete(&environment_path(workspace_root, slug, env_id), false)
        .await
}

pub async fn get_global_variables(
    file_ops: &dyn FileOps,
    workspace_root: &str,
) -> Result<Vec<EnvVar>, AppError> {
    match read_yaml::<Vec<EnvVar>>(file_ops, &global_vars_path(workspace_root)).await {
        Ok(v) => Ok(v),
        Err(_) => Ok(Vec::new()),
    }
}

pub async fn save_global_variables(
    file_ops: &dyn FileOps,
    workspace_root: &str,
    vars: &[EnvVar],
) -> Result<(), AppError> {
    write_yaml(file_ops, &global_vars_path(workspace_root), &vars.to_vec()).await
}

// ---------------------------------------------------------------------
// 请求
// ---------------------------------------------------------------------

/// 递归�?`requests/` 目录发现所有请求文件——文件夹只是路径前缀，不是独立实�?/// （docs/HTTP_DESKTOP_PLAN.md §3.3）。用显式栈而不�?`async fn` 自调用递归�?/// Rust �?`async fn` 不能直接自己递归（返回类型大小无限），显式栈规避这个限制�?/// 顺便也不用为这一处引�?`async-recursion` 之类的宏依赖�?pub async fn list_requests(
    file_ops: &dyn FileOps,
    workspace_root: &str,
    slug: &str,
) -> Result<Vec<RequestSummary>, AppError> {
    let root = requests_dir(workspace_root, slug);
    let mut out = Vec::new();
    let mut stack: Vec<Vec<String>> = vec![Vec::new()];
    while let Some(folder) = stack.pop() {
        let dir = if folder.is_empty() {
            root.clone()
        } else {
            format!("{root}/{}", folder.join("/"))
        };
        let entries = list_dir_or_empty(file_ops, &dir).await;
        for entry in entries {
            if entry.is_dir {
                let mut next = folder.clone();
                next.push(entry.name);
                stack.push(next);
            } else if entry.name.ends_with(".yaml") {
                let path = format!("{dir}/{}", entry.name);
                if let Ok(req) = read_yaml::<RequestDef>(file_ops, &path).await {
                    out.push(RequestSummary {
                        id: req.id,
                        name: req.name,
                        method: req.method,
                        folder: folder.clone(),
                    });
                }
            }
        }
    }
    out.sort_by(|a, b| (a.folder.join("/"), a.name.to_lowercase()).cmp(&(b.folder.join("/"), b.name.to_lowercase())));
    Ok(out)
}

fn request_path(workspace_root: &str, slug: &str, folder: &[String], id: &str) -> String {
    let dir = requests_dir(workspace_root, slug);
    if folder.is_empty() {
        format!("{dir}/{id}.yaml")
    } else {
        format!("{dir}/{}/{id}.yaml", folder.join("/"))
    }
}

/// 请求 id 是全局唯一的（uuid），但文件可能藏在任意深度的子文件夹下——找的时�?/// 没法直接拼路径，得先扫一遍拿�?`folder`。请求数量级（单集合几十到几百个）下
/// 这个代价可以接受，比维护一�?id -> folder"索引简单得多�?async fn find_request_path(
    file_ops: &dyn FileOps,
    workspace_root: &str,
    slug: &str,
    id: &str,
) -> Result<String, AppError> {
    let requests = list_requests(file_ops, workspace_root, slug).await?;
    let found = requests
        .into_iter()
        .find(|r| r.id == id)
        .ok_or_else(|| AppError::NotFound(format!("请求不存�? {id}")))?;
    Ok(request_path(workspace_root, slug, &found.folder, id))
}

pub async fn get_request(
    file_ops: &dyn FileOps,
    workspace_root: &str,
    slug: &str,
    id: &str,
) -> Result<RequestDef, AppError> {
    let path = find_request_path(file_ops, workspace_root, slug, id).await?;
    read_yaml(file_ops, &path).await
}

/// 新建请求——立刻分�?id 并落盘一个空模板（哪怕内容是空的），不等�?保存"�?/// 写文件。这是照�?SQL 桌面方案 §4.4 的既有结论：路径要在创建时就定好，未�?/// AI �?`ChangeStore::stage()` 按路径做身份匹配才能从第一次编辑起正常生成 diff
/// （脚本引�?AI 改动本轮未落地，但存储层先按这个约定做，后续�?AI 不用改格式）�?pub async fn create_request(
    file_ops: &dyn FileOps,
    workspace_root: &str,
    slug: &str,
    name: &str,
    folder: Vec<String>,
) -> Result<RequestDef, AppError> {
    let id = Uuid::new_v4().to_string();
    let req = RequestDef::new(id.clone(), name.to_string());
    let path = request_path(workspace_root, slug, &folder, &id);
    write_yaml(file_ops, &path, &req).await?;
    Ok(req)
}

pub async fn save_request(
    file_ops: &dyn FileOps,
    workspace_root: &str,
    slug: &str,
    req: &RequestDef,
) -> Result<(), AppError> {
    let path = find_request_path(file_ops, workspace_root, slug, &req.id).await?;
    write_yaml(file_ops, &path, req).await
}

pub async fn delete_request(
    file_ops: &dyn FileOps,
    workspace_root: &str,
    slug: &str,
    id: &str,
) -> Result<(), AppError> {
    let path = find_request_path(file_ops, workspace_root, slug, id).await?;
    file_ops.delete(&path, false).await
}

/// �?`import::parse_postman_collection`/`parse_openapi` 解析出来�?/// `ImportedCollection` 落盘成一个新集合——新建集合（�?slug 去重，和
/// `create_collection` 同一套逻辑�? 写集合元数据（名�?描述/变量�? 逐个请求
/// 直接写最终内容（不走"先建空模板再保存"两步，导入场景一次性给了完整内容，
/// 没必要拆成两次写盘）。整个导入没有部分失败时的回滚——某个请求写失败会让
/// 这次调用直接报错返回，前面已经写成功的文件和目录留在原地（比如权限问�?/// 导致某个请求写不进去）；调用方看到的�?导入失败"提示，可以清理这个半成的
/// 集合后重试，不做更复杂的"全有或全�?事务语义，导入是一次性操作，代价可接受�?pub async fn import_collection(
    file_ops: &dyn FileOps,
    workspace_root: &str,
    imported: super::import::ImportedCollection,
) -> Result<HttpCollectionSummary, AppError> {
    let base_slug = slugify(&imported.name);
    let existing = list_collections(file_ops, workspace_root).await?;
    let mut slug = base_slug.clone();
    let mut n = 2;
    while existing.iter().any(|c| c.slug == slug) {
        slug = format!("{base_slug}-{n}");
        n += 1;
    }
    file_ops.create_dir(&requests_dir(workspace_root, &slug)).await?;
    file_ops
        .create_dir(&environments_dir(workspace_root, &slug))
        .await?;
    let meta = HttpCollectionMeta {
        name: imported.name.clone(),
        description: imported.description.clone(),
        auth: AuthConfig::None,
        variables: imported.variables,
    };
    write_yaml(file_ops, &collection_meta_path(workspace_root, &slug), &meta).await?;
    let default_env = EnvironmentDef::new(Uuid::new_v4().to_string(), "默认环境".to_string());
    write_yaml(
        file_ops,
        &environment_path(workspace_root, &slug, &default_env.id),
        &default_env,
    )
    .await?;

    let mut request_count = 0usize;
    for imported_req in imported.requests {
        let id = Uuid::new_v4().to_string();
        let req = RequestDef {
            id: id.clone(),
            name: imported_req.name,
            method: imported_req.method,
            url: imported_req.url,
            params: imported_req.params,
            headers: imported_req.headers,
            auth: imported_req.auth,
            body: imported_req.body,
        };
        let path = request_path(workspace_root, &slug, &imported_req.folder, &id);
        write_yaml(file_ops, &path, &req).await?;
        request_count += 1;
    }

    Ok(HttpCollectionSummary {
        slug,
        name: meta.name,
        description: meta.description,
        request_count,
    })
}

/// 上层（`client.rs`）需要的完整变量上下文——把集合/全局两层拼好交给调用方，
/// 环境那一层由调用方按当前选中�?`environment_id` 单独读（避免这里重复�?/// 环境列表）�?pub async fn collection_variables(
    file_ops: &dyn FileOps,
    workspace_root: &str,
    slug: &str,
) -> Result<Vec<EnvVar>, AppError> {
    Ok(get_collection_meta(file_ops, workspace_root, slug)
        .await
        .map(|m| m.variables)
        .unwrap_or_default())
}
