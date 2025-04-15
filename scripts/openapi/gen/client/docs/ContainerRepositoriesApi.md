# \ContainerRepositoriesApi

All URIs are relative to *https://dev.api.miruml.com/internal/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_container_repository**](ContainerRepositoriesApi.md#get_container_repository) | **GET** /repositories/container/{container_repository_id} | Get a container repository
[**get_dockerhub_external_repositories**](ContainerRepositoriesApi.md#get_dockerhub_external_repositories) | **GET** /integrations/dockerhub/{dockerhub_integration_id}/repositories/external | Get the external repositories for a dockerhub integration
[**get_external_container_repositories**](ContainerRepositoriesApi.md#get_external_container_repositories) | **GET** /workspaces/{workspace_id}/repositories/container/external | Get the external repositories for a workspace
[**get_gar_external_repositories**](ContainerRepositoriesApi.md#get_gar_external_repositories) | **GET** /integrations/gar/{gar_integration_id}/repositories/external | Get the external repositories for a google artifact registry integration



## get_container_repository

> models::ContainerRepository get_container_repository(container_repository_id)
Get a container repository

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**container_repository_id** | **String** | The unique identifier of the container repository | [required] |

### Return type

[**models::ContainerRepository**](ContainerRepository.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_dockerhub_external_repositories

> models::ExternalContainerRepositoryList get_dockerhub_external_repositories(dockerhub_integration_id)
Get the external repositories for a dockerhub integration

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**dockerhub_integration_id** | **String** | The unique identifier of the dockerhub integration | [required] |

### Return type

[**models::ExternalContainerRepositoryList**](ExternalContainerRepositoryList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_external_container_repositories

> models::ExternalContainerRepositoryList get_external_container_repositories(workspace_id)
Get the external repositories for a workspace

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**workspace_id** | **String** | The unique identifier of the workspace | [required] |

### Return type

[**models::ExternalContainerRepositoryList**](ExternalContainerRepositoryList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_gar_external_repositories

> models::ExternalContainerRepositoryList get_gar_external_repositories(gar_integration_id)
Get the external repositories for a google artifact registry integration

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**gar_integration_id** | **String** | The unique identifier of the google artifact registry integration | [required] |

### Return type

[**models::ExternalContainerRepositoryList**](ExternalContainerRepositoryList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

