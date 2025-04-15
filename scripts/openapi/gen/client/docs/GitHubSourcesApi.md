# \GitHubSourcesApi

All URIs are relative to *https://dev.api.miruml.com/internal/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_git_hub_source_commits**](GitHubSourcesApi.md#get_git_hub_source_commits) | **GET** /sources/github/{github_source_id}/commits | Get the commits for a GitHub source



## get_git_hub_source_commits

> models::GitHubSourceCommits get_git_hub_source_commits(github_source_id)
Get the commits for a GitHub source

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**github_source_id** | **String** | The unique identifier of the github source | [required] |

### Return type

[**models::GitHubSourceCommits**](GitHubSourceCommits.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

