# \GitCommitsApi

All URIs are relative to *https://dev.api.miruml.com/internal/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_git_commits**](GitCommitsApi.md#get_git_commits) | **GET** /git_commits | Get the git commits for a workspace



## get_git_commits

> models::GitCommitList get_git_commits(offset, limit, order_by, expand_left_square_bracket_right_square_bracket, search)
Get the git commits for a workspace

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**offset** | Option<**i32**> | The offset to begin returning results from |  |[default to 0]
**limit** | Option<**i32**> | The number of items to return |  |[default to 10]
**order_by** | Option<[**Vec<models::GitCommitOrderBy>**](models::GitCommitOrderBy.md)> |  |  |
**expand_left_square_bracket_right_square_bracket** | Option<[**Vec<models::GitCommitExpand>**](models::GitCommitExpand.md)> |  |  |
**search** | Option<[**Vec<models::GitCommitSearch>**](models::GitCommitSearch.md)> |  |  |

### Return type

[**models::GitCommitList**](GitCommitList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

