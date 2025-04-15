# \WorkspacesApi

All URIs are relative to *https://dev.api.miruml.com/internal/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_workspace**](WorkspacesApi.md#get_workspace) | **GET** /workspace | Get a workspace
[**update_workspace**](WorkspacesApi.md#update_workspace) | **PUT** /workspace | Update a workspace



## get_workspace

> models::Workspace get_workspace()
Get a workspace

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::Workspace**](Workspace.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_workspace

> models::Workspace update_workspace(update_workspace_request)
Update a workspace

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**update_workspace_request** | [**UpdateWorkspaceRequest**](UpdateWorkspaceRequest.md) |  | [required] |

### Return type

[**models::Workspace**](Workspace.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

