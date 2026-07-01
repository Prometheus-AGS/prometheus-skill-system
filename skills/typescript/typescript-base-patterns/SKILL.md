---
license: MIT
name: typescript-base-patterns
version: '1.0.0'
description: >
  Canonical TypeScript 6 patterns for Prometheus AGS projects. Covers strict type
  discipline (no any, no @ts-ignore), discriminated unions for domain modeling,
  Result types for error handling, branded types for IDs, zod for runtime validation,
  and module organization. Applies to all TypeScript code: React, MCP servers,
  Mastra agents, and Node.js scripts.
language: typescript
metadata:
  tags: [typescript, patterns]
---

# TypeScript Base Patterns

## Strict Configuration

All Prometheus AGS TypeScript projects use `strict: true` with additional checks:

```json
{
  "compilerOptions": {
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "exactOptionalPropertyTypes": true,
    "noImplicitReturns": true,
    "noFallthroughCasesInSwitch": true
  }
}
```

`noUncheckedIndexedAccess` is the most impactful — it forces `arr[i]` to return
`T | undefined`, eliminating a large class of runtime errors.

## No `any` — Use `unknown` and Narrow

`any` disables type checking. Use `unknown` for truly unknown data and narrow with
type guards or zod parsing.

```ts
// ❌ Wrong
function processInput(data: any) { return data.value }

// ✅ Correct — narrow with zod
import { z } from 'zod'
const InputSchema = z.object({ value: z.string() })
function processInput(raw: unknown): string {
  const parsed = InputSchema.parse(raw) // throws on invalid input
  return parsed.value
}

// ✅ Correct — narrow with type guard
function isPost(value: unknown): value is Post {
  return typeof value === 'object' && value !== null && 'title' in value
}
```

## Discriminated Unions for Domain Types

Model domain variants as discriminated unions, not class hierarchies or string enums.

```ts
// Agent event types
type AgentEvent =
  | { type: 'thinking'; content: string }
  | { type: 'tool_call'; toolName: string; args: Record<string, unknown> }
  | { type: 'tool_result'; toolName: string; result: unknown }
  | { type: 'complete'; output: string }
  | { type: 'error'; message: string }

function handleEvent(event: AgentEvent) {
  switch (event.type) {
    case 'thinking': console.log(event.content); break
    case 'tool_call': dispatch(event.toolName, event.args); break
    case 'error':     logError(event.message); break
  }
}
```

TypeScript exhaustiveness checking will catch missed cases when you add a new variant.

## Result Types for Error Handling

Never throw in business logic. Return explicit `Result<T, E>` types.

```ts
type Result<T, E = Error> =
  | { ok: true; value: T }
  | { ok: false; error: E }

const ok = <T>(value: T): Result<T, never> => ({ ok: true, value })
const err = <E>(error: E): Result<never, E> => ({ ok: false, error })

async function fetchPost(id: string): Promise<Result<Post, 'not_found' | 'network_error'>> {
  try {
    const res = await fetch(`/api/posts/${id}`)
    if (res.status === 404) return err('not_found')
    if (!res.ok) return err('network_error')
    return ok(await res.json() as Post)
  } catch {
    return err('network_error')
  }
}

// Call site — exhaustive
const result = await fetchPost(id)
if (result.ok) {
  render(result.value)
} else {
  showError(result.error) // 'not_found' | 'network_error'
}
```

## Branded Types for IDs

Prevent mixing IDs of different entity types at compile time.

```ts
type Brand<T, B extends string> = T & { __brand: B }
type PostId = Brand<string, 'PostId'>
type UserId = Brand<string, 'UserId'>

const toPostId = (id: string): PostId => id as PostId
const toUserId = (id: string): UserId => id as UserId

function getPost(id: PostId): Promise<Post> { /* ... */ }

const uid = toUserId('user-123')
getPost(uid) // ✅ Type error — UserId is not assignable to PostId
```

## Zod for Runtime Validation

Use zod at all trust boundaries: API responses, user inputs, environment variables,
message queue payloads, MCP tool arguments.

```ts
import { z } from 'zod'

// Environment
const Env = z.object({
  DATABASE_URL: z.string().url(),
  API_KEY:      z.string().min(1),
  PORT:         z.coerce.number().default(3000),
})
export const env = Env.parse(process.env)

// MCP tool arguments
const RunInferenceArgs = z.object({
  prompt: z.string().min(1),
  model:  z.string().default('qwen2.5-7b'),
})
export type RunInferenceArgs = z.infer<typeof RunInferenceArgs>
```

## Module Organization

```
src/
  domain/       ← types, interfaces, zod schemas (no runtime deps)
  application/  ← use cases, service classes
  infrastructure/ ← API clients, DB adapters
  shared/       ← utilities: Result, branded types, cn()
```

Follow the CLEAN architecture skill for full layering rules.

## Forbidden Patterns

- `any` type — use `unknown` + narrowing or explicit types
- `// @ts-ignore` — fix the type error
- `as SomeType` without validation — use zod `parse()` at trust boundaries
- `interface` extending `any` — define explicit shapes
- `throw new Error()` in business logic — return `Result`
- Default exports from modules — use named exports
- `console.log` in production code — use a logger with structured output
