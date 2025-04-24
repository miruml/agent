# \ConfigSchemasApi

All URIs are relative to *https://configs.dev.api.miruml.com/internal/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**hash_config_schema**](ConfigSchemasApi.md#hash_config_schema) | **POST** /config_schemas/hash | Hash a config schema
[**hash_config_schema_serialized**](ConfigSchemasApi.md#hash_config_schema_serialized) | **POST** /config_schemas/hash/serialized | Hash a serialized config schema



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

