# \ComposeFileApi

All URIs are relative to *https://dev.api.miruml.com/internal/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**verify_compose_file**](ComposeFileApi.md#verify_compose_file) | **POST** /workspaces/{workspace_id}/compose/verify | Verify a compose file



## verify_compose_file

> models::ComposeFileVerification verify_compose_file(workspace_id, compose_file_verification_request)
Verify a compose file

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**workspace_id** | **String** | The unique identifier of the workspace | [required] |
**compose_file_verification_request** | [**ComposeFileVerificationRequest**](ComposeFileVerificationRequest.md) |  | [required] |

### Return type

[**models::ComposeFileVerification**](ComposeFileVerification.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

