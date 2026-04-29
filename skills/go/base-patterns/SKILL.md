---
license: MIT
name: base-patterns
version: '1.0.0'
description: >
  Canonical Go 1.22 patterns for Prometheus AGS projects. Covers error handling with
  %w wrapping, context propagation, interface-based dependency injection, structured
  logging with slog, module layout (cmd/internal/pkg), table-driven tests, and
  idiomatic Go patterns used across Prometheus infrastructure tooling and MCP servers.
language: go
---

# Go Base Patterns

## Error Handling

Go errors are values. Wrap with `%w` to preserve the chain. Never use `panic` in
library code. Never swallow errors with `_`.

```go
import (
    "errors"
    "fmt"
)

// Sentinel errors for comparison
var (
    ErrNotFound      = errors.New("not found")
    ErrUnauthorized  = errors.New("unauthorized")
)

// Wrap with context
func loadConfig(path string) (*Config, error) {
    data, err := os.ReadFile(path)
    if err != nil {
        return nil, fmt.Errorf("loading config from %s: %w", path, err)
    }
    // ...
}

// Check specific error
config, err := loadConfig(path)
if errors.Is(err, os.ErrNotExist) {
    return defaultConfig(), nil
}
if err != nil {
    return nil, fmt.Errorf("config initialization: %w", err)
}
```

## Context Propagation

Every function that does I/O, makes a network call, or can be cancelled takes
`context.Context` as its first parameter.

```go
func (c *Client) FetchPost(ctx context.Context, id string) (*Post, error) {
    req, err := http.NewRequestWithContext(ctx, http.MethodGet, c.baseURL+"/posts/"+id, nil)
    if err != nil {
        return nil, fmt.Errorf("building request: %w", err)
    }
    resp, err := c.http.Do(req)
    // ...
}
```

Never store context in a struct. Pass it through every call chain.

## Interface-Based Dependency Injection

Define interfaces at the point of use, not at the point of implementation.
Keep interfaces small — prefer single-method interfaces.

```go
// Define in the package that uses it
type PostRepository interface {
    FindByID(ctx context.Context, id string) (*Post, error)
    List(ctx context.Context, filter PostFilter) ([]*Post, error)
    Save(ctx context.Context, post *Post) error
}

// Implement separately (infrastructure)
type PostgresPostRepository struct { db *sql.DB }
func (r *PostgresPostRepository) FindByID(ctx context.Context, id string) (*Post, error) { /* ... */ }

// Compose at startup (cmd/server/main.go)
func main() {
    db := mustConnectDB()
    repo := &infrastructure.PostgresPostRepository{DB: db}
    svc := application.NewPostService(repo)
    handler := interface_layer.NewPostHandler(svc)
    // ...
}
```

## Structured Logging with slog

Use `log/slog` (stdlib since 1.21). Always include relevant context fields.
Never use `fmt.Println` for operational logging.

```go
import "log/slog"

logger := slog.New(slog.NewJSONHandler(os.Stdout, &slog.HandlerOptions{
    Level: slog.LevelInfo,
}))

// With context fields
logger.Info("inference complete",
    slog.String("model", model),
    slog.Int("tokens", tokenCount),
    slog.Duration("latency", elapsed),
)

logger.Error("inference failed",
    slog.String("model", model),
    slog.Any("error", err),
)
```

## Module Layout

```
my-service/
├── cmd/
│   └── server/
│       └── main.go          ← composition root, dependency wiring
├── internal/                ← private to this module
│   ├── domain/
│   │   ├── post.go          ← domain entities
│   │   └── repository.go    ← interfaces
│   ├── application/
│   │   └── post_service.go  ← use cases
│   └── infrastructure/
│       └── postgres_repo.go ← concrete implementations
├── pkg/                     ← exported shared packages (if any)
├── go.mod
└── go.sum
```

Use `internal/` to enforce that packages are not imported from outside the module.

## Table-Driven Tests

```go
func TestFetchPost(t *testing.T) {
    tests := []struct {
        name    string
        id      string
        want    *Post
        wantErr error
    }{
        {name: "found", id: "post-1", want: &Post{ID: "post-1", Title: "Hello"}, wantErr: nil},
        {name: "not found", id: "missing", want: nil, wantErr: ErrNotFound},
    }

    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            repo := &mockPostRepo{posts: seedPosts()}
            got, err := repo.FindByID(context.Background(), tt.id)
            if !errors.Is(err, tt.wantErr) {
                t.Errorf("got error %v, want %v", err, tt.wantErr)
            }
            // compare got vs tt.want
        })
    }
}
```

## HTTP Handler Pattern

```go
// Handler wraps the service — thin translation layer
type PostHandler struct {
    svc PostService
    log *slog.Logger
}

func (h *PostHandler) GetPost(w http.ResponseWriter, r *http.Request) {
    id := r.PathValue("id") // Go 1.22 stdlib routing
    post, err := h.svc.FetchPost(r.Context(), id)
    if err != nil {
        if errors.Is(err, ErrNotFound) {
            http.Error(w, "not found", http.StatusNotFound)
            return
        }
        h.log.Error("fetch post failed", slog.Any("error", err))
        http.Error(w, "internal error", http.StatusInternalServerError)
        return
    }
    w.Header().Set("Content-Type", "application/json")
    json.NewEncoder(w).Encode(post)
}
```

## Forbidden Patterns

- `panic()` in library code — return errors
- `interface{}` — use `any` (Go 1.18+ alias) or concrete types
- Storing `context.Context` in a struct
- Ignoring errors: `result, _ = fn()` — handle or propagate
- `init()` with side effects — use explicit initialization in `main`
- Global mutable state — inject via interfaces
