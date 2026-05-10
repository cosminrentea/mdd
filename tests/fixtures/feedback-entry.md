---
type: feedback
topic: testing
---

# Integration Tests Must Use Real Dependencies

Integration tests must hit actual backing services, not mocks.

**Why:** Previous incident where mocked tests passed but production migration failed due to schema drift.

**How to apply:** Use ephemeral containers for integration tests. Reserve mocks for unit tests of pure business logic only.

## Examples

### Good

```java
@IntegrationTest
@EphemeralContainers
class IngestionIT {
    @Container
    static DocStoreContainer store = new DocStoreContainer("docstore:7");
}
```

### Bad

```java
@MockBean
DocStoreTemplate storeTemplate; // Never for integration tests
```
