# \ConfigSchemasApi

All URIs are relative to *https://dev.api.miruml.com/internal/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_config_schema**](ConfigSchemasApi.md#create_config_schema) | **POST** /config_schemas | Create a config schema
[**delete_config_schema**](ConfigSchemasApi.md#delete_config_schema) | **DELETE** /config_schemas/{config_schema_id} | Delete a config schema
[**get_config_schema**](ConfigSchemasApi.md#get_config_schema) | **GET** /config_schemas/{config_schema_id} | Get a config schema
[**hash_config_schema**](ConfigSchemasApi.md#hash_config_schema) | **POST** /config_schemas/hash | Hash a config schema
[**list_config_schemas**](ConfigSchemasApi.md#list_config_schemas) | **GET** /config_schemas | List the config schemas for a workspace
[**render_config_schema**](ConfigSchemasApi.md#render_config_schema) | **POST** /config_schemas/{config_schema_id}/render | Render a config schema



## create_config_schema

> models::ConfigSchema create_config_schema(create_config_schema_request, expand)
Create a config schema

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_config_schema_request** | [**CreateConfigSchemaRequest**](CreateConfigSchemaRequest.md) |  | [required] |
**expand** | Option<[**Vec<models::ConfigSchemaExpand>**](models::ConfigSchemaExpand.md)> |  |  |

### Return type

[**models::ConfigSchema**](ConfigSchema.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_config_schema

> models::ConfigSchema delete_config_schema(config_schema_id, expand)
Delete a config schema

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**config_schema_id** | **String** | The unique identifier of the config schema | [required] |
**expand** | Option<[**Vec<models::ConfigSchemaExpand>**](models::ConfigSchemaExpand.md)> |  |  |

### Return type

[**models::ConfigSchema**](ConfigSchema.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_config_schema

> models::ConfigSchema get_config_schema(config_schema_id, expand)
Get a config schema

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**config_schema_id** | **String** | The unique identifier of the config schema | [required] |
**expand** | Option<[**Vec<models::ConfigSchemaExpand>**](models::ConfigSchemaExpand.md)> |  |  |

### Return type

[**models::ConfigSchema**](ConfigSchema.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## hash_config_schema

> models::SchemaDigestResponse hash_config_schema(hash_schema_request)
Hash a config schema

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**hash_schema_request** | [**HashSchemaRequest**](HashSchemaRequest.md) |  | [required] |

### Return type

[**models::SchemaDigestResponse**](SchemaDigestResponse.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_config_schemas

> models::ConfigSchemaList list_config_schemas(workspace_id, offset, limit, order_by, search, expand)
List the config schemas for a workspace

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**workspace_id** | **String** | The unique identifier of the workspace | [required] |
**offset** | Option<**i32**> | The offset to begin returning results from |  |[default to 0]
**limit** | Option<**i32**> | The number of items to return |  |[default to 10]
**order_by** | Option<[**Vec<models::ConfigSchemaOrderBy>**](models::ConfigSchemaOrderBy.md)> |  |  |
**search** | Option<[**Vec<models::ConfigSchemaSearch>**](models::ConfigSchemaSearch.md)> |  |  |
**expand** | Option<[**Vec<models::ConfigSchemaExpand>**](models::ConfigSchemaExpand.md)> |  |  |

### Return type

[**models::ConfigSchemaList**](ConfigSchemaList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## render_config_schema

> models::RenderedConfigSchema render_config_schema(config_schema_id, render_config_schema_request, expand)
Render a config schema

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**config_schema_id** | **String** | The unique identifier of the config schema | [required] |
**render_config_schema_request** | [**RenderConfigSchemaRequest**](RenderConfigSchemaRequest.md) |  | [required] |
**expand** | Option<[**Vec<models::RenderedConfigSchemaExpand>**](models::RenderedConfigSchemaExpand.md)> |  |  |

### Return type

[**models::RenderedConfigSchema**](RenderedConfigSchema.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

