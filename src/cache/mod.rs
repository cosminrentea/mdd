// Cache module -- deferred.
// At 51ms for 320 files (release build), caching is not needed.
// If vault grows to 3000+ files and latency becomes noticeable, add
// redb + postcard caching here with mtime-based invalidation.
