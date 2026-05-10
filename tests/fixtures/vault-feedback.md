---
type: feedback
topic: testing
---

# Integration Tests Must Use Real DB

Integration tests must hit a real database, not mocks.

**Why:** Prior incident where mock/prod divergence masked a broken migration.

**How to apply:** Use TestContainers or LocalStack for integration tests. Reserve mocks for unit tests of business logic only.

## Examples

### Good

```java
@SpringBootTest
@Testcontainers
class PersistenceIT {
    @Container
    static MongoDBContainer mongo = new MongoDBContainer("mongo:7");
}
```

### Bad

```java
@MockBean
MongoTemplate mongoTemplate; // Never for integration tests
```
