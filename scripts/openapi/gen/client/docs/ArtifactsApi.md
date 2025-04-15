# \ArtifactsApi

All URIs are relative to *https://dev.api.miruml.com/internal/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_git_hub_source_artifact**](ArtifactsApi.md#create_git_hub_source_artifact) | **POST** /sources/github/{github_source_id}/artifacts | Create an artifact from a github source
[**create_registry_source_artifact**](ArtifactsApi.md#create_registry_source_artifact) | **POST** /sources/registry/{registry_source_id}/artifacts | Create an artifact from a registry source
[**get_artifact**](ArtifactsApi.md#get_artifact) | **GET** /artifacts/{artifact_id} | Get an artifact
[**get_artifact_build_logs**](ArtifactsApi.md#get_artifact_build_logs) | **GET** /artifacts/{artifact_id}/logs/build | Get the build logs for an artifact
[**get_artifact_files**](ArtifactsApi.md#get_artifact_files) | **GET** /artifacts/{artifact_id}/files | Get the files for an artifact



## create_git_hub_source_artifact

> create_git_hub_source_artifact(github_source_id, create_git_hub_source_artifact)
Create an artifact from a github source

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**github_source_id** | **String** | The unique identifier of the github source | [required] |
**create_git_hub_source_artifact** | [**CreateGitHubSourceArtifact**](CreateGitHubSourceArtifact.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_registry_source_artifact

> models::Artifact create_registry_source_artifact(registry_source_id, create_registry_source_artifact)
Create an artifact from a registry source

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**registry_source_id** | **String** | The unique identifier of the registry source | [required] |
**create_registry_source_artifact** | [**CreateRegistrySourceArtifact**](CreateRegistrySourceArtifact.md) |  | [required] |

### Return type

[**models::Artifact**](Artifact.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_artifact

> models::Artifact get_artifact(artifact_id)
Get an artifact

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**artifact_id** | **String** | The unique identifier of the artifact | [required] |

### Return type

[**models::Artifact**](Artifact.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_artifact_build_logs

> models::ArtifactBuildLogs get_artifact_build_logs(artifact_id)
Get the build logs for an artifact

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**artifact_id** | **String** | The unique identifier of the artifact | [required] |

### Return type

[**models::ArtifactBuildLogs**](ArtifactBuildLogs.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_artifact_files

> models::ArtifactFiles get_artifact_files(artifact_id)
Get the files for an artifact

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**artifact_id** | **String** | The unique identifier of the artifact | [required] |

### Return type

[**models::ArtifactFiles**](ArtifactFiles.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

