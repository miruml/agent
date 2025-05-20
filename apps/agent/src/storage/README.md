Why use Arc<T> for the cache but not for the sync boolean? 

Arc (Reference Counter) allows for a multiple ownership paradigm without duplicating the memory allocation (like clone() would). The primary benefit of this is that the stg
struct can be ready from without having to reference the stg struct itself. This simplifies a lot of the code and reasoning about the code.

We use Arc for the cache because it is large and (potentially) expensive to clone. However, the sync boolean is small and cheap to copy so we use it directly. This allows us to efficiently use the Arc<T> cache without having to clone it and without having to reference the stg struct itself.