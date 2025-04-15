# \RegistrySourcesApi

All URIs are relative to *https://dev.api.miruml.com/internal/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_registry_source**](RegistrySourcesApi.md#create_registry_source) | **POST** /workspaces/{workspace_id}/sources/registry | Create a registry source
[**delete_registry_source**](RegistrySourcesApi.md#delete_registry_source) | **DELETE** /sources/registry/{registry_source_id} | Delete a registry source
[**get_registry_source**](RegistrySourcesApi.md#get_registry_source) | **GET** /sources/registry/{registry_source_id} | Get a registry source
[**get_registry_source_compose_file**](RegistrySourcesApi.md#get_registry_source_compose_file) | **GET** /sources/registry/{registry_source_id}/compose_file | Get the compose file for a registry source
[**get_registry_sources**](RegistrySourcesApi.md#get_registry_sources) | **GET** /workspaces/{workspace_id}/sources/registry | Get the registry sources for a workspace
[**update_registry_source**](RegistrySourcesApi.md#update_registry_source) | **PATCH** /sources/registry/{registry_source_id} | Update a registry source



## create_registry_source

> models::RegistrySource create_registry_source(workspace_id, create_registry_source_request)
Create a registry source

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**workspace_id** | **String** | The unique identifier of the workspace | [required] |
**create_registry_source_request** | [**CreateRegistrySourceRequest**](CreateRegistrySourceRequest.md) |  | [required] |

### Return type

[**models::RegistrySource**](RegistrySource.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_registry_source

> models::RegistrySource delete_registry_source(registry_source_id)
Delete a registry source

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**registry_source_id** | **String** | The unique identifier of the registry source | [required] |

### Return type

[**models::RegistrySource**](RegistrySource.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_registry_source

> models::RegistrySource get_registry_source(registry_source_id)
Get a registry source

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**registry_source_id** | **String** | The unique identifier of the registry source | [required] |

### Return type

[**models::RegistrySource**](RegistrySource.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_registry_source_compose_file

> models::ComposeFile get_registry_source_compose_file(registry_source_id)
Get the compose file for a registry source

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**registry_source_id** | **String** | The unique identifier of the registry source | [required] |

### Return type

[**models::ComposeFile**](ComposeFile.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_registry_sources

> models::RegistrySourceList get_registry_sources(workspace_id)
Get the registry sources for a workspace

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**workspace_id** | **String** | The unique identifier of the workspace | [required] |

### Return type

[**models::RegistrySourceList**](RegistrySourceList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_registry_source

> models::RegistrySource update_registry_source(registry_source_id, update_registry_source_request)
Update a registry source

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**registry_source_id** | **String** | The unique identifier of the registry source | [required] |
**update_registry_source_request** | [**UpdateRegistrySourceRequest**](UpdateRegistrySourceRequest.md) |  | [required] |

### Return type

[**models::RegistrySource**](RegistrySource.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

