# PatchCommentList

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**object** | **String** |  | 
**total_count** | **i64** | The total number of items in the list. By default the total count is not returned. The total count must be expanded (using expand[]=total_count) to get the total number of items in the list. | 
**limit** | **i32** | The maximum number of items to return. A limit of 15 with an offset of 0 returns items 1-15. | [default to 10]
**offset** | **i32** | The offset of the items to return. An offset of 10 with a limit of 10 returns items 11-20. | [default to 0]
**has_more** | **bool** | True if there are more items in the list to return. False if there are no more items to return. | 
**data** | [**Vec<models::PatchComment>**](PatchComment.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


