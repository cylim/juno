use crate::controllers::store::get_admin_controllers;
use crate::controllers::store::{
    delete_controllers as delete_controllers_store, get_controllers,
    set_controllers as set_controllers_store,
};
use crate::db::store::{
    count_docs_store, delete_doc_store, delete_docs_store, get_doc_store, list_docs_store,
    set_doc_store,
};
use crate::db::types::interface::{DelDoc, SetDoc};
use crate::db::types::state::{Doc, DocContext, DocUpsert};
use crate::hooks::{
    invoke_on_delete_asset, invoke_on_delete_doc, invoke_on_delete_many_assets,
    invoke_on_delete_many_docs, invoke_on_set_doc, invoke_on_set_many_docs, invoke_upload_asset,
};
use crate::memory::{get_memory_upgrades, init_stable_state, STATE};
use crate::rules::store::{
    del_rule_db, del_rule_storage, get_rules_db, get_rules_storage, set_rule_db, set_rule_storage,
};
use crate::rules::types::interface::{DelRule, SetRule};
use crate::rules::types::rules::Rule;
use crate::storage::constants::{RESPONSE_STATUS_CODE_200, RESPONSE_STATUS_CODE_405};
use crate::storage::http::response::{
    build_asset_response, build_redirect_response, error_response,
};
use crate::storage::http::types::{
    HttpRequest, HttpResponse, StreamingCallbackHttpResponse, StreamingCallbackToken,
};
use crate::storage::http::utils::create_token;
use crate::storage::routing::get_routing;
use crate::storage::store::{
    commit_batch_store, count_assets_store, create_batch_store, create_chunk_store,
    delete_asset_store, delete_assets_store, delete_domain_store, get_asset_store,
    get_config_store as get_storage_config, get_content_chunks_store, get_custom_domains_store,
    get_public_asset_store, init_certified_assets_store, list_assets_store,
    set_config_store as set_storage_config, set_domain_store,
};
use crate::storage::types::domain::{CustomDomains, DomainName};
use crate::storage::types::http_request::{
    Routing, RoutingDefault, RoutingRedirect, RoutingRewrite,
};
use crate::storage::types::interface::{
    AssetNoContent, CommitBatch, InitAssetKey, InitUploadResult, UploadChunk, UploadChunkResult,
};
use crate::storage::types::state::FullPath;
use crate::storage::types::store::Asset;
use crate::types::core::{CollectionKey, Key};
use crate::types::interface::{Config, RulesType};
use crate::types::list::ListParams;
use crate::types::list::ListResults;
use crate::types::memory::Memory;
use crate::types::state::{HeapState, RuntimeState, State};
use ciborium::{from_reader, into_writer};
use ic_cdk::api::call::arg_data;
use ic_cdk::api::{caller, trap};
use ic_stable_structures::writer::Writer;
#[allow(unused)]
use ic_stable_structures::Memory as _;
use junobuild_shared::constants::MAX_NUMBER_OF_SATELLITE_CONTROLLERS;
use junobuild_shared::controllers::{
    assert_max_number_of_controllers, assert_no_anonymous_controller, init_controllers,
};
use junobuild_shared::types::interface::{DeleteControllersArgs, SegmentArgs, SetControllersArgs};
use junobuild_shared::types::state::{ControllerScope, Controllers};
use std::mem;

pub fn init() {
    let call_arg = arg_data::<(Option<SegmentArgs>,)>().0;
    let SegmentArgs { controllers } = call_arg.unwrap();

    let heap = HeapState {
        controllers: init_controllers(&controllers),
        ..HeapState::default()
    };

    STATE.with(|state| {
        *state.borrow_mut() = State {
            stable: init_stable_state(),
            heap,
            runtime: RuntimeState::default(),
        };
    });
}

pub fn pre_upgrade() {
    // Serialize the state using CBOR.
    let mut state_bytes = vec![];
    STATE
        .with(|s| into_writer(&*s.borrow(), &mut state_bytes))
        .expect("Failed to encode the state of the satellite in pre_upgrade hook.");

    // Write the length of the serialized bytes to memory, followed by the by the bytes themselves.
    let len = state_bytes.len() as u32;
    let mut memory = get_memory_upgrades();
    let mut writer = Writer::new(&mut memory, 0);
    writer.write(&len.to_le_bytes()).unwrap();
    writer.write(&state_bytes).unwrap()
}

pub fn post_upgrade() {
    // The memory offset is 4 bytes because that's the length we used in pre_upgrade to store the length of the memory data for the upgrade.
    // https://github.com/dfinity/stable-structures/issues/104
    const OFFSET: usize = mem::size_of::<u32>();

    let memory: Memory = get_memory_upgrades();

    // Read the length of the state bytes.
    let mut state_len_bytes = [0; OFFSET];
    memory.read(0, &mut state_len_bytes);
    let state_len = u32::from_le_bytes(state_len_bytes) as usize;

    // Read the bytes
    let mut state_bytes = vec![0; state_len];
    memory.read(u64::try_from(OFFSET).unwrap(), &mut state_bytes);

    // Deserialize and set the state.
    let state = from_reader(&*state_bytes)
        .expect("Failed to decode the state of the satellite in post_upgrade hook.");
    STATE.with(|s| *s.borrow_mut() = state);

    init_certified_assets_store();
}

///
/// Db
///

pub fn set_doc(collection: CollectionKey, key: Key, doc: SetDoc) -> Doc {
    let caller = caller();

    let result = set_doc_store(caller, collection, key, doc);

    match result {
        Ok(doc) => {
            invoke_on_set_doc(&caller, &doc);

            doc.data.after
        }
        Err(error) => trap(&error),
    }
}

pub fn get_doc(collection: CollectionKey, key: Key) -> Option<Doc> {
    let caller = caller();

    let result = get_doc_store(caller, collection, key);

    match result {
        Ok(value) => value,
        Err(error) => trap(&error),
    }
}

pub fn del_doc(collection: CollectionKey, key: Key, doc: DelDoc) {
    let caller = caller();

    let deleted_doc = delete_doc_store(caller, collection, key, doc).unwrap_or_else(|e| trap(&e));

    invoke_on_delete_doc(&caller, &deleted_doc);
}

pub fn list_docs(collection: CollectionKey, filter: ListParams) -> ListResults<Doc> {
    let caller = caller();

    let result = list_docs_store(caller, collection, &filter);

    match result {
        Ok(value) => value,
        Err(error) => trap(&error),
    }
}

pub fn get_many_docs(docs: Vec<(CollectionKey, Key)>) -> Vec<(Key, Option<Doc>)> {
    docs.iter()
        .map(|(collection, key)| {
            let doc = get_doc(collection.clone(), key.clone());
            (key.clone(), doc.clone())
        })
        .collect()
}

pub fn set_many_docs(docs: Vec<(CollectionKey, Key, SetDoc)>) -> Vec<(Key, Doc)> {
    let caller = caller();

    let mut hook_payload: Vec<DocContext<DocUpsert>> = Vec::new();
    let mut results: Vec<(Key, Doc)> = Vec::new();

    for (collection, key, doc) in docs {
        let result =
            set_doc_store(caller, collection, key.clone(), doc).unwrap_or_else(|e| trap(&e));

        results.push((result.key.clone(), result.data.after.clone()));

        hook_payload.push(result);
    }

    invoke_on_set_many_docs(&caller, &hook_payload);

    results
}

pub fn del_many_docs(docs: Vec<(CollectionKey, Key, DelDoc)>) {
    let caller = caller();

    let mut results: Vec<DocContext<Option<Doc>>> = Vec::new();

    for (collection, key, doc) in docs {
        let deleted_doc =
            delete_doc_store(caller, collection, key.clone(), doc).unwrap_or_else(|e| trap(&e));
        results.push(deleted_doc);
    }

    invoke_on_delete_many_docs(&caller, &results);
}

pub fn del_docs(collection: CollectionKey) {
    let result = delete_docs_store(&collection);

    match result {
        Ok(_) => (),
        Err(error) => trap(&["Documents cannot be deleted: ", &error].join("")),
    }
}

pub fn count_docs(collection: CollectionKey) -> usize {
    let result = count_docs_store(&collection);

    match result {
        Ok(value) => value,
        Err(error) => trap(&error),
    }
}

/// Rules

pub fn list_rules(rules_type: RulesType) -> Vec<(CollectionKey, Rule)> {
    match rules_type {
        RulesType::Db => get_rules_db(),
        RulesType::Storage => get_rules_storage(),
    }
}

pub fn set_rule(rules_type: RulesType, collection: CollectionKey, rule: SetRule) {
    match rules_type {
        RulesType::Db => set_rule_db(collection, rule).unwrap_or_else(|e| trap(&e)),
        RulesType::Storage => set_rule_storage(collection, rule).unwrap_or_else(|e| trap(&e)),
    }
}

pub fn del_rule(rules_type: RulesType, collection: CollectionKey, rule: DelRule) {
    match rules_type {
        RulesType::Db => del_rule_db(collection, rule).unwrap_or_else(|e| trap(&e)),
        RulesType::Storage => del_rule_storage(collection, rule).unwrap_or_else(|e| trap(&e)),
    }
}

///
/// Controllers
///

pub fn set_controllers(
    SetControllersArgs {
        controllers,
        controller,
    }: SetControllersArgs,
) -> Controllers {
    match controller.scope {
        ControllerScope::Write => {}
        ControllerScope::Admin => {
            let max_controllers = assert_max_number_of_controllers(
                &get_admin_controllers(),
                &controllers,
                MAX_NUMBER_OF_SATELLITE_CONTROLLERS,
            );

            if let Err(err) = max_controllers {
                trap(&err)
            }
        }
    }

    assert_no_anonymous_controller(&controllers).unwrap_or_else(|e| trap(&e));

    set_controllers_store(&controllers, &controller);
    get_controllers()
}

pub fn del_controllers(
    DeleteControllersArgs { controllers }: DeleteControllersArgs,
) -> Controllers {
    delete_controllers_store(&controllers);
    get_controllers()
}

pub fn list_controllers() -> Controllers {
    get_controllers()
}

///
/// Config
///

pub fn set_config(config: Config) {
    set_storage_config(&config.storage);
}

pub fn get_config() -> Config {
    let storage = get_storage_config();
    Config { storage }
}

///
/// Custom domains
///

pub fn list_custom_domains() -> CustomDomains {
    get_custom_domains_store()
}

pub fn set_custom_domain(domain_name: DomainName, bn_id: Option<String>) {
    set_domain_store(&domain_name, &bn_id).unwrap_or_else(|e| trap(&e));
}

pub fn del_custom_domain(domain_name: DomainName) {
    delete_domain_store(&domain_name).unwrap_or_else(|e| trap(&e));
}

///
/// Http
///

pub fn http_request(
    HttpRequest {
        method,
        url,
        headers: req_headers,
        body: _,
        certificate_version,
    }: HttpRequest,
) -> HttpResponse {
    if method != "GET" {
        return error_response(RESPONSE_STATUS_CODE_405, "Method Not Allowed.".to_string());
    }

    let result = get_routing(url, true);

    match result {
        Ok(routing) => match routing {
            Routing::Default(RoutingDefault { url, asset }) => build_asset_response(
                url,
                req_headers,
                certificate_version,
                asset,
                None,
                RESPONSE_STATUS_CODE_200,
            ),
            Routing::Rewrite(RoutingRewrite {
                url,
                asset,
                source,
                status_code,
            }) => build_asset_response(
                url,
                req_headers,
                certificate_version,
                asset,
                Some(source),
                status_code,
            ),
            Routing::Redirect(RoutingRedirect {
                url,
                redirect,
                iframe,
            }) => build_redirect_response(url, certificate_version, &redirect, &iframe),
        },
        Err(err) => error_response(
            RESPONSE_STATUS_CODE_405,
            ["Permission denied. Cannot perform this operation. ", err].join(""),
        ),
    }
}

pub fn http_request_streaming_callback(
    StreamingCallbackToken {
        token,
        headers,
        index,
        sha256: _,
        full_path,
        encoding_type,
        memory: _,
    }: StreamingCallbackToken,
) -> StreamingCallbackHttpResponse {
    let asset = get_public_asset_store(full_path, token);

    match asset {
        Some((asset, memory)) => {
            let encoding = asset.encodings.get(&encoding_type);

            match encoding {
                Some(encoding) => {
                    let body = get_content_chunks_store(encoding, index, &memory);

                    match body {
                        Some(body) => StreamingCallbackHttpResponse {
                            token: create_token(
                                &asset.key,
                                index,
                                encoding,
                                &encoding_type,
                                &headers,
                                &memory,
                            ),
                            body: body.clone(),
                        },
                        None => trap("Streamed chunks not found."),
                    }
                }
                None => trap("Streamed asset encoding not found."),
            }
        }
        None => trap("Streamed asset not found."),
    }
}

//
// Storage
//

pub fn init_asset_upload(init: InitAssetKey) -> InitUploadResult {
    let caller = caller();
    let result = create_batch_store(caller, init);

    match result {
        Ok(batch_id) => InitUploadResult { batch_id },
        Err(error) => trap(&error),
    }
}

pub fn upload_asset_chunk(chunk: UploadChunk) -> UploadChunkResult {
    let caller = caller();

    let result = create_chunk_store(caller, chunk);

    match result {
        Ok(chunk_id) => UploadChunkResult { chunk_id },
        Err(error) => trap(error),
    }
}

pub fn commit_asset_upload(commit: CommitBatch) {
    let caller = caller();

    let asset = commit_batch_store(caller, commit).unwrap_or_else(|e| trap(&e));

    invoke_upload_asset(&caller, &asset);
}

pub fn list_assets(collection: CollectionKey, filter: ListParams) -> ListResults<AssetNoContent> {
    let caller = caller();

    let result = list_assets_store(caller, &collection, &filter);

    match result {
        Ok(result) => result,
        Err(error) => trap(&["Assets cannot be listed: ".to_string(), error].join("")),
    }
}

pub fn del_asset(collection: CollectionKey, full_path: FullPath) {
    let caller = caller();

    let result = delete_asset_store(caller, &collection, full_path);

    match result {
        Ok(asset) => invoke_on_delete_asset(&caller, &asset),
        Err(error) => trap(&["Asset cannot be deleted: ", &error].join("")),
    }
}

pub fn del_many_assets(assets: Vec<(CollectionKey, String)>) {
    let caller = caller();

    let mut results: Vec<Option<Asset>> = Vec::new();

    for (collection, full_path) in assets {
        let deleted_asset =
            delete_asset_store(caller, &collection, full_path).unwrap_or_else(|e| trap(&e));
        results.push(deleted_asset);
    }

    invoke_on_delete_many_assets(&caller, &results);
}

pub fn del_assets(collection: CollectionKey) {
    let result = delete_assets_store(&collection);

    match result {
        Ok(_) => (),
        Err(error) => trap(&["Assets cannot be deleted: ", &error].join("")),
    }
}

pub fn count_assets(collection: CollectionKey) -> usize {
    let result = count_assets_store(&collection);

    match result {
        Ok(value) => value,
        Err(error) => trap(&error),
    }
}

pub fn get_asset(collection: CollectionKey, full_path: FullPath) -> Option<AssetNoContent> {
    let caller = caller();

    let result = get_asset_store(caller, &collection, full_path);

    match result {
        Ok(asset) => asset.map(|asset| AssetNoContent::from(&asset)),
        Err(error) => trap(&error),
    }
}

pub fn get_many_assets(
    assets: Vec<(CollectionKey, FullPath)>,
) -> Vec<(FullPath, Option<AssetNoContent>)> {
    assets
        .iter()
        .map(|(collection, full_path)| {
            let asset = get_asset(collection.clone(), full_path.clone());
            (full_path.clone(), asset.clone())
        })
        .collect()
}
