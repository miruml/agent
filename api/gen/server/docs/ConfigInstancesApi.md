# \ConfigInstancesApi

All URIs are relative to *http://localhost/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_deployed_config_instance**](ConfigInstancesApi.md#get_deployed_config_instance) | **GET** /config_instances/deployed | Get the deployed config instance



## get_deployed_config_instance

> models::ConfigInstance get_deployed_config_instance(config_schema_digest, config_type_slug)
Get the deployed config instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**config_schema_digest** | **String** | The digest of the config schema | [required] |
**config_type_slug** | **String** | The slug of the config type | [required] |

### Return type

[**models::ConfigInstance**](ConfigInstance.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

