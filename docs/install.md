# Test Cases

## Success
- Clean install
    - Download the package
    - From local package
- Idempotent install
    - Download the package
    - From local package
- Upgrade
    - Download the package
    - From local package

## Failures
- Version is not supported
- Version does not exist
- Activation token not provided
    - Miru agent is still restarted

### From Local Package
- Package does not exist
- Invalid mimetype
- Invalid debian package name
- Incompatible architecture