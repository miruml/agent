# \JobRunsApi

All URIs are relative to *https://dev.api.miruml.com/internal/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_job_run**](JobRunsApi.md#get_job_run) | **GET** /jobs/runs/{job_run_id} | Get a job run
[**get_job_runs**](JobRunsApi.md#get_job_runs) | **GET** /jobs/{job_id}/runs | Get all job runs for a job



## get_job_run

> models::JobRun get_job_run(job_run_id)
Get a job run

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_run_id** | **String** | The unique identifier of the job run | [required] |

### Return type

[**models::JobRun**](JobRun.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_job_runs

> models::JobRunList get_job_runs(job_id)
Get all job runs for a job

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_id** | **String** | The unique identifier of the job | [required] |

### Return type

[**models::JobRunList**](JobRunList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

