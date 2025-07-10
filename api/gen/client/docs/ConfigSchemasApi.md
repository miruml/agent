# \ConfigSchemasApi

All URIs are relative to *https://configs.dev.api.miruml.com/agent/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**hash_config_schema**](ConfigSchemasApi.md#hash_config_schema) | **POST** /config_schemas/hash | Hash a config schema
[**hash_config_schema_serialized**](ConfigSchemasApi.md#hash_config_schema_serialized) | **POST** /config_schemas/hash/serialized | Hash a serialized config schema
[**list_config_schemas**](ConfigSchemasApi.md#list_config_schemas) | **GET** /config_schemas | List the config schemas for a workspace



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


## hash_config_schema_serialized

> models::SchemaDigestResponse hash_config_schema_serialized(hash_schema_serialized_request)
Hash a serialized config schema

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**hash_schema_serialized_request** | [**HashSchemaSerializedRequest**](HashSchemaSerializedRequest.md) |  | [required] |

### Return type

[**models::SchemaDigestResponse**](SchemaDigestResponse.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_config_schemas

> models::ConfigSchemaList list_config_schemas(offset, limit, order_by, search, expand_left_square_bracket_right_square_bracket)
List the config schemas for a workspace

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**offset** | Option<**i32**> | The offset to begin returning results from |  |[default to 0]
**limit** | Option<**i32**> | The number of items to return |  |[default to 10]
**order_by** | Option<[**Vec<models::ConfigSchemaOrderBy>**](models::ConfigSchemaOrderBy.md)> |  |  |
**search** | Option<[**Vec<models::ConfigSchemaSearch>**](models::ConfigSchemaSearch.md)> |  |  |
**expand_left_square_bracket_right_square_bracket** | Option<[**Vec<models::ConfigSchemaExpand>**](models::ConfigSchemaExpand.md)> |  |  |

### Return type

[**models::ConfigSchemaList**](ConfigSchemaList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

