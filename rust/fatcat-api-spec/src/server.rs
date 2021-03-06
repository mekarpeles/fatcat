#![allow(unused_extern_crates)]
extern crate bodyparser;
extern crate chrono;
extern crate iron;
extern crate router;
extern crate serde_ignored;
extern crate urlencoded;
extern crate uuid;

use self::iron::prelude::*;
use self::iron::url::percent_encoding::percent_decode;
use self::iron::{modifiers, status, BeforeMiddleware};
use self::router::Router;
use self::urlencoded::UrlEncodedQuery;
use futures::future;
use futures::Future;
use futures::{stream, Stream};
use hyper;
use hyper::header::{ContentType, Headers};
use mimetypes;

use serde_json;

#[allow(unused_imports)]
use std::collections::{BTreeMap, HashMap};
use std::io::Error;
#[allow(unused_imports)]
use swagger;

#[allow(unused_imports)]
use std::collections::BTreeSet;

pub use swagger::auth::Authorization;
use swagger::auth::{AuthData, Scopes};
use swagger::{ApiError, Context, XSpanId};

#[allow(unused_imports)]
use models;
use {
    AcceptEditgroupResponse, Api, CreateContainerBatchResponse, CreateContainerResponse, CreateCreatorBatchResponse, CreateCreatorResponse, CreateEditgroupResponse, CreateFileBatchResponse,
    CreateFileResponse, CreateReleaseBatchResponse, CreateReleaseResponse, CreateWorkBatchResponse, CreateWorkResponse, DeleteContainerResponse, DeleteCreatorResponse, DeleteFileResponse,
    DeleteReleaseResponse, DeleteWorkResponse, GetChangelogEntryResponse, GetChangelogResponse, GetContainerHistoryResponse, GetContainerResponse, GetCreatorHistoryResponse,
    GetCreatorReleasesResponse, GetCreatorResponse, GetEditgroupResponse, GetEditorChangelogResponse, GetEditorResponse, GetFileHistoryResponse, GetFileResponse, GetReleaseFilesResponse,
    GetReleaseHistoryResponse, GetReleaseResponse, GetStatsResponse, GetWorkHistoryResponse, GetWorkReleasesResponse, GetWorkResponse, LookupContainerResponse, LookupCreatorResponse,
    LookupFileResponse, LookupReleaseResponse, UpdateContainerResponse, UpdateCreatorResponse, UpdateFileResponse, UpdateReleaseResponse, UpdateWorkResponse,
};

header! { (Warning, "Warning") => [String] }

/// Create a new router for `Api`
pub fn router<T>(api: T) -> Router
where
    T: Api + Send + Sync + Clone + 'static,
{
    let mut router = Router::new();
    add_routes(&mut router, api);
    router
}

/// Add routes for `Api` to a provided router.
///
/// Note that these routes are added straight onto the router. This means that if the router
/// already has a route for an endpoint which clashes with those provided by this API, then the
/// old route will be lost.
///
/// It is generally a bad idea to add routes in this way to an existing router, which may have
/// routes on it for other APIs. Distinct APIs should be behind distinct paths to encourage
/// separation of interfaces, which this function does not enforce. APIs should not overlap.
///
/// Alternative approaches include:
///
/// - generate an `iron::middleware::Handler` (usually a `router::Router` or
///   `iron::middleware::chain`) for each interface, and add those handlers inside an existing
///   router, mounted at different paths - so the interfaces are separated by path
/// - use a different instance of `iron::Iron` for each interface - so the interfaces are
///   separated by the address/port they listen on
///
/// This function exists to allow legacy code, which doesn't separate its APIs properly, to make
/// use of this crate.
#[deprecated(note = "APIs should not overlap - only for use in legacy code.")]
pub fn route<T>(router: &mut Router, api: T)
where
    T: Api + Send + Sync + Clone + 'static,
{
    add_routes(router, api)
}

/// Add routes for `Api` to a provided router
fn add_routes<T>(router: &mut Router, api: T)
where
    T: Api + Send + Sync + Clone + 'static,
{
    let api_clone = api.clone();
    router.post(
        "/v0/container",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup = query_params.get("editgroup").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::ContainerEntity> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    }).map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.create_container(param_entity, param_editgroup, context).wait() {
                    Ok(rsp) => match rsp {
                        CreateContainerResponse::CreatedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CONTAINER_CREATED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateContainerResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CONTAINER_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateContainerResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CONTAINER_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateContainerResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CONTAINER_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "CreateContainer",
    );

    let api_clone = api.clone();
    router.post(
        "/v0/container/batch",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_autoaccept = query_params.get("autoaccept").and_then(|list| list.first()).and_then(|x| x.parse::<bool>().ok());
                let param_editgroup = query_params.get("editgroup").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity_list = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity_list = if let Some(param_entity_list_raw) = param_entity_list {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_list_raw);

                    let param_entity_list: Option<Vec<models::ContainerEntity>> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    }).map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - doesn't match schema: {}", e))))?;

                    param_entity_list
                } else {
                    None
                };
                let param_entity_list = param_entity_list.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity_list".to_string())))?;

                match api.create_container_batch(param_entity_list.as_ref(), param_autoaccept, param_editgroup, context).wait() {
                    Ok(rsp) => match rsp {
                        CreateContainerBatchResponse::CreatedEntities(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CONTAINER_BATCH_CREATED_ENTITIES.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateContainerBatchResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CONTAINER_BATCH_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateContainerBatchResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CONTAINER_BATCH_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateContainerBatchResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CONTAINER_BATCH_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "CreateContainerBatch",
    );

    let api_clone = api.clone();
    router.delete(
        "/v0/container/:id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup = query_params.get("editgroup").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.delete_container(param_id, param_editgroup, context).wait() {
                    Ok(rsp) => match rsp {
                        DeleteContainerResponse::DeletedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_CONTAINER_DELETED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteContainerResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_CONTAINER_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteContainerResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_CONTAINER_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteContainerResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_CONTAINER_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "DeleteContainer",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/container/:id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_expand = query_params.get("expand").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.get_container(param_id, param_expand, context).wait() {
                    Ok(rsp) => match rsp {
                        GetContainerResponse::FoundEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CONTAINER_FOUND_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetContainerResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CONTAINER_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetContainerResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CONTAINER_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetContainerResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CONTAINER_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetContainer",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/container/:id/history",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_limit = query_params.get("limit").and_then(|list| list.first()).and_then(|x| x.parse::<i64>().ok());

                match api.get_container_history(param_id, param_limit, context).wait() {
                    Ok(rsp) => match rsp {
                        GetContainerHistoryResponse::FoundEntityHistory(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CONTAINER_HISTORY_FOUND_ENTITY_HISTORY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetContainerHistoryResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CONTAINER_HISTORY_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetContainerHistoryResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CONTAINER_HISTORY_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetContainerHistoryResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CONTAINER_HISTORY_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetContainerHistory",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/container/lookup",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_issnl = query_params
                    .get("issnl")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter issnl".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter issnl was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter issnl - doesn't match schema: {}", e))))?;

                match api.lookup_container(param_issnl, context).wait() {
                    Ok(rsp) => match rsp {
                        LookupContainerResponse::FoundEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_CONTAINER_FOUND_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        LookupContainerResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_CONTAINER_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        LookupContainerResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_CONTAINER_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        LookupContainerResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_CONTAINER_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "LookupContainer",
    );

    let api_clone = api.clone();
    router.put(
        "/v0/container/:id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup = query_params.get("editgroup").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::ContainerEntity> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    }).map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.update_container(param_id, param_entity, param_editgroup, context).wait() {
                    Ok(rsp) => match rsp {
                        UpdateContainerResponse::UpdatedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_CONTAINER_UPDATED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateContainerResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_CONTAINER_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateContainerResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_CONTAINER_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateContainerResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_CONTAINER_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "UpdateContainer",
    );

    let api_clone = api.clone();
    router.post(
        "/v0/creator",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup = query_params.get("editgroup").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::CreatorEntity> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    }).map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.create_creator(param_entity, param_editgroup, context).wait() {
                    Ok(rsp) => match rsp {
                        CreateCreatorResponse::CreatedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CREATOR_CREATED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateCreatorResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CREATOR_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateCreatorResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CREATOR_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateCreatorResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CREATOR_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "CreateCreator",
    );

    let api_clone = api.clone();
    router.post(
        "/v0/creator/batch",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_autoaccept = query_params.get("autoaccept").and_then(|list| list.first()).and_then(|x| x.parse::<bool>().ok());
                let param_editgroup = query_params.get("editgroup").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity_list = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity_list = if let Some(param_entity_list_raw) = param_entity_list {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_list_raw);

                    let param_entity_list: Option<Vec<models::CreatorEntity>> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    }).map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - doesn't match schema: {}", e))))?;

                    param_entity_list
                } else {
                    None
                };
                let param_entity_list = param_entity_list.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity_list".to_string())))?;

                match api.create_creator_batch(param_entity_list.as_ref(), param_autoaccept, param_editgroup, context).wait() {
                    Ok(rsp) => match rsp {
                        CreateCreatorBatchResponse::CreatedEntities(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CREATOR_BATCH_CREATED_ENTITIES.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateCreatorBatchResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CREATOR_BATCH_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateCreatorBatchResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CREATOR_BATCH_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateCreatorBatchResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CREATOR_BATCH_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "CreateCreatorBatch",
    );

    let api_clone = api.clone();
    router.delete(
        "/v0/creator/:id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup = query_params.get("editgroup").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.delete_creator(param_id, param_editgroup, context).wait() {
                    Ok(rsp) => match rsp {
                        DeleteCreatorResponse::DeletedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_CREATOR_DELETED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteCreatorResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_CREATOR_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteCreatorResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_CREATOR_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteCreatorResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_CREATOR_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "DeleteCreator",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/creator/:id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_expand = query_params.get("expand").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.get_creator(param_id, param_expand, context).wait() {
                    Ok(rsp) => match rsp {
                        GetCreatorResponse::FoundEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_FOUND_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetCreatorResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetCreatorResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetCreatorResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetCreator",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/creator/:id/history",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_limit = query_params.get("limit").and_then(|list| list.first()).and_then(|x| x.parse::<i64>().ok());

                match api.get_creator_history(param_id, param_limit, context).wait() {
                    Ok(rsp) => match rsp {
                        GetCreatorHistoryResponse::FoundEntityHistory(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_HISTORY_FOUND_ENTITY_HISTORY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetCreatorHistoryResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_HISTORY_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetCreatorHistoryResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_HISTORY_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetCreatorHistoryResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_HISTORY_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetCreatorHistory",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/creator/:id/releases",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                match api.get_creator_releases(param_id, context).wait() {
                    Ok(rsp) => match rsp {
                        GetCreatorReleasesResponse::Found(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_RELEASES_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetCreatorReleasesResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_RELEASES_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetCreatorReleasesResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_RELEASES_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetCreatorReleasesResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_RELEASES_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetCreatorReleases",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/creator/lookup",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_orcid = query_params
                    .get("orcid")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter orcid".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter orcid was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter orcid - doesn't match schema: {}", e))))?;

                match api.lookup_creator(param_orcid, context).wait() {
                    Ok(rsp) => match rsp {
                        LookupCreatorResponse::FoundEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_CREATOR_FOUND_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        LookupCreatorResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_CREATOR_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        LookupCreatorResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_CREATOR_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        LookupCreatorResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_CREATOR_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "LookupCreator",
    );

    let api_clone = api.clone();
    router.put(
        "/v0/creator/:id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup = query_params.get("editgroup").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::CreatorEntity> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    }).map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.update_creator(param_id, param_entity, param_editgroup, context).wait() {
                    Ok(rsp) => match rsp {
                        UpdateCreatorResponse::UpdatedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_CREATOR_UPDATED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateCreatorResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_CREATOR_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateCreatorResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_CREATOR_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateCreatorResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_CREATOR_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "UpdateCreator",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/editor/:id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                match api.get_editor(param_id, context).wait() {
                    Ok(rsp) => match rsp {
                        GetEditorResponse::Found(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITOR_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditorResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITOR_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditorResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITOR_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditorResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITOR_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetEditor",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/editor/:id/changelog",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                match api.get_editor_changelog(param_id, context).wait() {
                    Ok(rsp) => match rsp {
                        GetEditorChangelogResponse::Found(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITOR_CHANGELOG_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditorChangelogResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITOR_CHANGELOG_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditorChangelogResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITOR_CHANGELOG_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditorChangelogResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITOR_CHANGELOG_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetEditorChangelog",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/stats",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_more = query_params.get("more").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.get_stats(param_more, context).wait() {
                    Ok(rsp) => match rsp {
                        GetStatsResponse::Success(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_STATS_SUCCESS.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetStatsResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_STATS_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetStats",
    );

    let api_clone = api.clone();
    router.post(
        "/v0/editgroup/:id/accept",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                match api.accept_editgroup(param_id, context).wait() {
                    Ok(rsp) => match rsp {
                        AcceptEditgroupResponse::MergedSuccessfully(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::ACCEPT_EDITGROUP_MERGED_SUCCESSFULLY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        AcceptEditgroupResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::ACCEPT_EDITGROUP_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        AcceptEditgroupResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::ACCEPT_EDITGROUP_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        AcceptEditgroupResponse::EditConflict(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(409), body_string));
                            response.headers.set(ContentType(mimetypes::responses::ACCEPT_EDITGROUP_EDIT_CONFLICT.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        AcceptEditgroupResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::ACCEPT_EDITGROUP_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "AcceptEditgroup",
    );

    let api_clone = api.clone();
    router.post(
        "/v0/editgroup",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_editgroup = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter editgroup - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_editgroup = if let Some(param_editgroup_raw) = param_editgroup {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_editgroup_raw);

                    let param_editgroup: Option<models::Editgroup> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    }).map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter editgroup - doesn't match schema: {}", e))))?;

                    param_editgroup
                } else {
                    None
                };
                let param_editgroup = param_editgroup.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter editgroup".to_string())))?;

                match api.create_editgroup(param_editgroup, context).wait() {
                    Ok(rsp) => match rsp {
                        CreateEditgroupResponse::SuccessfullyCreated(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_EDITGROUP_SUCCESSFULLY_CREATED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateEditgroupResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_EDITGROUP_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateEditgroupResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_EDITGROUP_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "CreateEditgroup",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/changelog",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_limit = query_params.get("limit").and_then(|list| list.first()).and_then(|x| x.parse::<i64>().ok());

                match api.get_changelog(param_limit, context).wait() {
                    Ok(rsp) => match rsp {
                        GetChangelogResponse::Success(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CHANGELOG_SUCCESS.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetChangelogResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CHANGELOG_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetChangelog",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/changelog/:id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                match api.get_changelog_entry(param_id, context).wait() {
                    Ok(rsp) => match rsp {
                        GetChangelogEntryResponse::FoundChangelogEntry(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CHANGELOG_ENTRY_FOUND_CHANGELOG_ENTRY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetChangelogEntryResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CHANGELOG_ENTRY_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetChangelogEntryResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CHANGELOG_ENTRY_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetChangelogEntry",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/editgroup/:id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                match api.get_editgroup(param_id, context).wait() {
                    Ok(rsp) => match rsp {
                        GetEditgroupResponse::Found(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITGROUP_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditgroupResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITGROUP_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditgroupResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITGROUP_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditgroupResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITGROUP_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetEditgroup",
    );

    let api_clone = api.clone();
    router.post(
        "/v0/file",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup = query_params.get("editgroup").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::FileEntity> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    }).map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.create_file(param_entity, param_editgroup, context).wait() {
                    Ok(rsp) => match rsp {
                        CreateFileResponse::CreatedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILE_CREATED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateFileResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILE_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateFileResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILE_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateFileResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILE_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "CreateFile",
    );

    let api_clone = api.clone();
    router.post(
        "/v0/file/batch",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_autoaccept = query_params.get("autoaccept").and_then(|list| list.first()).and_then(|x| x.parse::<bool>().ok());
                let param_editgroup = query_params.get("editgroup").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity_list = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity_list = if let Some(param_entity_list_raw) = param_entity_list {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_list_raw);

                    let param_entity_list: Option<Vec<models::FileEntity>> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    }).map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - doesn't match schema: {}", e))))?;

                    param_entity_list
                } else {
                    None
                };
                let param_entity_list = param_entity_list.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity_list".to_string())))?;

                match api.create_file_batch(param_entity_list.as_ref(), param_autoaccept, param_editgroup, context).wait() {
                    Ok(rsp) => match rsp {
                        CreateFileBatchResponse::CreatedEntities(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILE_BATCH_CREATED_ENTITIES.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateFileBatchResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILE_BATCH_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateFileBatchResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILE_BATCH_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateFileBatchResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILE_BATCH_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "CreateFileBatch",
    );

    let api_clone = api.clone();
    router.delete(
        "/v0/file/:id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup = query_params.get("editgroup").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.delete_file(param_id, param_editgroup, context).wait() {
                    Ok(rsp) => match rsp {
                        DeleteFileResponse::DeletedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_FILE_DELETED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteFileResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_FILE_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteFileResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_FILE_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteFileResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_FILE_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "DeleteFile",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/file/:id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_expand = query_params.get("expand").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.get_file(param_id, param_expand, context).wait() {
                    Ok(rsp) => match rsp {
                        GetFileResponse::FoundEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILE_FOUND_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFileResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILE_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFileResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILE_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFileResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILE_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetFile",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/file/:id/history",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_limit = query_params.get("limit").and_then(|list| list.first()).and_then(|x| x.parse::<i64>().ok());

                match api.get_file_history(param_id, param_limit, context).wait() {
                    Ok(rsp) => match rsp {
                        GetFileHistoryResponse::FoundEntityHistory(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILE_HISTORY_FOUND_ENTITY_HISTORY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFileHistoryResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILE_HISTORY_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFileHistoryResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILE_HISTORY_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFileHistoryResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILE_HISTORY_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetFileHistory",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/file/lookup",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_sha1 = query_params
                    .get("sha1")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter sha1".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter sha1 was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter sha1 - doesn't match schema: {}", e))))?;

                match api.lookup_file(param_sha1, context).wait() {
                    Ok(rsp) => match rsp {
                        LookupFileResponse::FoundEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_FILE_FOUND_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        LookupFileResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_FILE_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        LookupFileResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_FILE_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        LookupFileResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_FILE_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "LookupFile",
    );

    let api_clone = api.clone();
    router.put(
        "/v0/file/:id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup = query_params.get("editgroup").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::FileEntity> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    }).map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.update_file(param_id, param_entity, param_editgroup, context).wait() {
                    Ok(rsp) => match rsp {
                        UpdateFileResponse::UpdatedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_FILE_UPDATED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateFileResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_FILE_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateFileResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_FILE_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateFileResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_FILE_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "UpdateFile",
    );

    let api_clone = api.clone();
    router.post(
        "/v0/release",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup = query_params.get("editgroup").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::ReleaseEntity> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    }).map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.create_release(param_entity, param_editgroup, context).wait() {
                    Ok(rsp) => match rsp {
                        CreateReleaseResponse::CreatedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_RELEASE_CREATED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateReleaseResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_RELEASE_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateReleaseResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_RELEASE_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateReleaseResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_RELEASE_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "CreateRelease",
    );

    let api_clone = api.clone();
    router.post(
        "/v0/release/batch",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_autoaccept = query_params.get("autoaccept").and_then(|list| list.first()).and_then(|x| x.parse::<bool>().ok());
                let param_editgroup = query_params.get("editgroup").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity_list = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity_list = if let Some(param_entity_list_raw) = param_entity_list {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_list_raw);

                    let param_entity_list: Option<Vec<models::ReleaseEntity>> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    }).map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - doesn't match schema: {}", e))))?;

                    param_entity_list
                } else {
                    None
                };
                let param_entity_list = param_entity_list.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity_list".to_string())))?;

                match api.create_release_batch(param_entity_list.as_ref(), param_autoaccept, param_editgroup, context).wait() {
                    Ok(rsp) => match rsp {
                        CreateReleaseBatchResponse::CreatedEntities(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_RELEASE_BATCH_CREATED_ENTITIES.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateReleaseBatchResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_RELEASE_BATCH_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateReleaseBatchResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_RELEASE_BATCH_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateReleaseBatchResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_RELEASE_BATCH_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "CreateReleaseBatch",
    );

    let api_clone = api.clone();
    router.post(
        "/v0/work",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup = query_params.get("editgroup").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::WorkEntity> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    }).map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.create_work(param_entity, param_editgroup, context).wait() {
                    Ok(rsp) => match rsp {
                        CreateWorkResponse::CreatedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_WORK_CREATED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateWorkResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_WORK_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateWorkResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_WORK_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateWorkResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_WORK_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "CreateWork",
    );

    let api_clone = api.clone();
    router.delete(
        "/v0/release/:id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup = query_params.get("editgroup").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.delete_release(param_id, param_editgroup, context).wait() {
                    Ok(rsp) => match rsp {
                        DeleteReleaseResponse::DeletedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_RELEASE_DELETED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteReleaseResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_RELEASE_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteReleaseResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_RELEASE_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteReleaseResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_RELEASE_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "DeleteRelease",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/release/:id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_expand = query_params.get("expand").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.get_release(param_id, param_expand, context).wait() {
                    Ok(rsp) => match rsp {
                        GetReleaseResponse::FoundEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_FOUND_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetRelease",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/release/:id/files",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                match api.get_release_files(param_id, context).wait() {
                    Ok(rsp) => match rsp {
                        GetReleaseFilesResponse::Found(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_FILES_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseFilesResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_FILES_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseFilesResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_FILES_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseFilesResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_FILES_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetReleaseFiles",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/release/:id/history",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_limit = query_params.get("limit").and_then(|list| list.first()).and_then(|x| x.parse::<i64>().ok());

                match api.get_release_history(param_id, param_limit, context).wait() {
                    Ok(rsp) => match rsp {
                        GetReleaseHistoryResponse::FoundEntityHistory(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_HISTORY_FOUND_ENTITY_HISTORY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseHistoryResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_HISTORY_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseHistoryResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_HISTORY_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseHistoryResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_HISTORY_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetReleaseHistory",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/release/lookup",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_doi = query_params
                    .get("doi")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter doi".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter doi was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter doi - doesn't match schema: {}", e))))?;

                match api.lookup_release(param_doi, context).wait() {
                    Ok(rsp) => match rsp {
                        LookupReleaseResponse::FoundEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_RELEASE_FOUND_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        LookupReleaseResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_RELEASE_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        LookupReleaseResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_RELEASE_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        LookupReleaseResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_RELEASE_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "LookupRelease",
    );

    let api_clone = api.clone();
    router.put(
        "/v0/release/:id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup = query_params.get("editgroup").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::ReleaseEntity> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    }).map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.update_release(param_id, param_entity, param_editgroup, context).wait() {
                    Ok(rsp) => match rsp {
                        UpdateReleaseResponse::UpdatedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_RELEASE_UPDATED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateReleaseResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_RELEASE_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateReleaseResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_RELEASE_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateReleaseResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_RELEASE_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "UpdateRelease",
    );

    let api_clone = api.clone();
    router.post(
        "/v0/work/batch",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_autoaccept = query_params.get("autoaccept").and_then(|list| list.first()).and_then(|x| x.parse::<bool>().ok());
                let param_editgroup = query_params.get("editgroup").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity_list = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity_list = if let Some(param_entity_list_raw) = param_entity_list {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_list_raw);

                    let param_entity_list: Option<Vec<models::WorkEntity>> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    }).map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - doesn't match schema: {}", e))))?;

                    param_entity_list
                } else {
                    None
                };
                let param_entity_list = param_entity_list.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity_list".to_string())))?;

                match api.create_work_batch(param_entity_list.as_ref(), param_autoaccept, param_editgroup, context).wait() {
                    Ok(rsp) => match rsp {
                        CreateWorkBatchResponse::CreatedEntities(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_WORK_BATCH_CREATED_ENTITIES.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateWorkBatchResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_WORK_BATCH_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateWorkBatchResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_WORK_BATCH_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateWorkBatchResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_WORK_BATCH_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "CreateWorkBatch",
    );

    let api_clone = api.clone();
    router.delete(
        "/v0/work/:id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup = query_params.get("editgroup").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.delete_work(param_id, param_editgroup, context).wait() {
                    Ok(rsp) => match rsp {
                        DeleteWorkResponse::DeletedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_WORK_DELETED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteWorkResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_WORK_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteWorkResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_WORK_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteWorkResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_WORK_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "DeleteWork",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/work/:id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_expand = query_params.get("expand").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.get_work(param_id, param_expand, context).wait() {
                    Ok(rsp) => match rsp {
                        GetWorkResponse::FoundEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_FOUND_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWorkResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWorkResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWorkResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetWork",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/work/:id/history",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_limit = query_params.get("limit").and_then(|list| list.first()).and_then(|x| x.parse::<i64>().ok());

                match api.get_work_history(param_id, param_limit, context).wait() {
                    Ok(rsp) => match rsp {
                        GetWorkHistoryResponse::FoundEntityHistory(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_HISTORY_FOUND_ENTITY_HISTORY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWorkHistoryResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_HISTORY_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWorkHistoryResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_HISTORY_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWorkHistoryResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_HISTORY_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetWorkHistory",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/work/:id/releases",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                match api.get_work_releases(param_id, context).wait() {
                    Ok(rsp) => match rsp {
                        GetWorkReleasesResponse::Found(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_RELEASES_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWorkReleasesResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_RELEASES_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWorkReleasesResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_RELEASES_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWorkReleasesResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_RELEASES_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetWorkReleases",
    );

    let api_clone = api.clone();
    router.put(
        "/v0/work/:id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup = query_params.get("editgroup").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::WorkEntity> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    }).map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.update_work(param_id, param_entity, param_editgroup, context).wait() {
                    Ok(rsp) => match rsp {
                        UpdateWorkResponse::UpdatedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_WORK_UPDATED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateWorkResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_WORK_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateWorkResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_WORK_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateWorkResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_WORK_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "UpdateWork",
    );
}

/// Middleware to extract authentication data from request
pub struct ExtractAuthData;

impl BeforeMiddleware for ExtractAuthData {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        Ok(())
    }
}
