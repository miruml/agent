# \JobsApi

All URIs are relative to *https://dev.api.miruml.com/internal/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_job**](JobsApi.md#create_job) | **POST** /jobs | Create a job
[**create_job_from_template**](JobsApi.md#create_job_from_template) | **POST** /jobs/templates/{job_template_id}/jobs | Create a job from a template
[**get_job**](JobsApi.md#get_job) | **GET** /jobs/{job_id} | Get a job
[**get_jobs**](JobsApi.md#get_jobs) | **GET** /workspaces/{workspace_id}/jobs | Get all jobs for a workspace



## create_job

> models::Job create_job(create_job_request)
Create a job

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_job_request** | [**CreateJobRequest**](CreateJobRequest.md) |  | [required] |

### Return type

[**models::Job**](Job.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_job_from_template

> models::Job create_job_from_template(job_template_id, create_job_from_template_request)
Create a job from a template

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_template_id** | **String** | The unique identifier of the job template | [required] |
**create_job_from_template_request** | [**CreateJobFromTemplateRequest**](CreateJobFromTemplateRequest.md) |  | [required] |

### Return type

[**models::Job**](Job.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_job

> models::Job get_job(job_id)
Get a job

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_id** | **String** | The unique identifier of the job | [required] |

### Return type

[**models::Job**](Job.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_jobs

> models::BaseJobList get_jobs(workspace_id)
Get all jobs for a workspace

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**workspace_id** | **String** | The unique identifier of the workspace | [required] |

### Return type

[**models::BaseJobList**](BaseJobList.md)

### Authorization

[ClerkAuth](../README.md#ClerkAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

