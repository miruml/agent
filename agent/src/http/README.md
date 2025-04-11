## Responsibility
The `HTTPClient` is responsible for interacting with the backend server for CRUD operations. 

It is NOT responsible for storing those changes in the storage module. It is not responsible for ensuring the storage module and the server data are synced. It is just used for sending and receiving data using an HTTP protocol with the server.

## Module Interactions
### Filesys (Okay)
Using the `filesys` module is expected. Although such use should be fairly limited. The primary application of the `filesys` module in the context of the `HTTPClient` is for downloading and uploading files. However, considering how foundational of a module `filesys` is, its use is generally encouraged throughout the codebase.

### Models (Discouraged)
The HTTP client should have extremely limited information about the `models` module. The data structures known to the web client methods should not involve the `models` module. However, it is both useful and convenient to provide transformation methods from a model, such as an Artifact, to the Artifact payload used in a POST artifact request. These are expected but should not be methods on the `HTTPClient`, but methods on the payload objects themselves or just a simple function which takes input the appropriate model (and maybe some other parameters) and outputs the payload.

### Storage (Avoid At All Costs)
The HTTP client should avoid the use of the storage module at all costs. Despite key information, such as the HTTP client token, timeout settings, the device id, etc. living in the storage module, the HTTP client DOES NOT need to know about the storage module to achieve its goals. Therefore, such information should be passed in by the module using the HTTP client.


## Design Patterns

### Singleton(ish)
According to the `reqwest` crate (the library used for HTTP requests),

> The Client holds a connection pool internally, so it is advised that you create one and reuse it. - [source](https://docs.rs/reqwest/latest/reqwest/struct.Client.html)

Thus, we adopt a [singleton](https://refactoring.guru/design-patterns/singleton)-like  pattern to initialize the `reqwest` client within the `HTTPClient`. This is not strictly a singleton pattern since we are cloning the `reqwest` client but it's very close to it IMO. You could arguably classify this as the [flyweight](https://refactoring.guru/design-patterns/flyweight), since the `reqwest` library is reusing the same resources for connection pooling and such but our application doesn't know about this functionality so I liken it more to the singleton pattern.