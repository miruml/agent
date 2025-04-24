# \ClientsApi

All URIs are relative to *https://configs.dev.api.miruml.com/internal/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**activate_client**](ClientsApi.md#activate_client) | **POST** /clients/{client_id}/activate | Activate a client
[**issue_client_token**](ClientsApi.md#issue_client_token) | **POST** /clients/{client_id}/issue_token | Issue a client token



## activate_client

> models::Client activate_client(client_id, activate_client_request)
Activate a client

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**client_id** | **String** | The unique identifier of the client | [required] |
**activate_client_request** | Option<[**ActivateClientRequest**](ActivateClientRequest.md)> |  |  |

### Return type

[**models::Client**](Client.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## issue_client_token

> models::IssueClientTokenResponse issue_client_token(client_id, issue_client_token_request)
Issue a client token

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**client_id** | **String** | The unique identifier of the client | [required] |
**issue_client_token_request** | Option<[**IssueClientTokenRequest**](IssueClientTokenRequest.md)> |  |  |

### Return type

[**models::IssueClientTokenResponse**](IssueClientTokenResponse.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

