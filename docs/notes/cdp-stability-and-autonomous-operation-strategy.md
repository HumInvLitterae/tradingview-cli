# CDP stability and autonomous-operation strategy

Status: long-term strategy note, not an executable implementation plan.

This note records the direction behind the v0.29 transport work and possible
later Desktop-operation improvements. Each promoted implementation slice needs
its own self-contained ExecPlan, focused review, acceptance evidence, and
commit. The first promoted slice is transport measurement and failure taxonomy;
later sections remain candidates until their prerequisite evidence is green.

## Motivation

Desktop-backed `tv` commands are short-lived processes. Most commands discover
a TradingView Desktop target over the local CDP HTTP endpoint, open a new
WebSocket, perform one bounded operation, and exit. This model is simple and
fail-closed, but transient target-list refusal, WebSocket handshake delay, or a
target endpoint that changes during a tab reload can make an otherwise valid
command fail. Repeating a command also pays process, discovery, connection, and
evaluation costs again.

The long-term objective is not merely lower latency. An agent should be able to
distinguish failures, choose a safe recovery action, wait for explicit
post-conditions, and avoid repeating a mutation whose first result is unknown.
Measurement must precede behavior changes so the project improves observed
problems rather than introducing retries or background infrastructure by
assumption.

## Durable boundaries

- A mutation is never retried automatically after a CDP method may have been
  dispatched. Unknown outcomes remain fail-closed.
- Pre-dispatch target listing, target selection, and WebSocket connection are
  distinct from re-running a command after method dispatch.
- Every retry, wait, probe, and shared-process lifetime must have one absolute
  bound that deterministic tests can enforce.
- Diagnostics must not expose raw target IDs, WebSocket URLs, Runtime payloads,
  account-local identifiers, credentials, or machine-specific paths.
- Internal diagnostics and public JSON contracts are separate layers. Internal
  transport stages may be detailed; public fields require an explicit,
  reviewed mapping to a smaller stable vocabulary.
- The process-per-command path remains the default. A shared process is not
  smuggled past the project's daemon exclusion by giving it a TTL. Any broker
  proposal requires an explicit policy decision and a separate feasibility
  plan.
- No broker or session is auto-spawned. No hidden transport fallback may make
  the connection owner or observed performance unreadable.
- Desktop-free and Desktop-backed source boundaries, mutation post-checks, and
  existing command contracts remain explicit.

## Ordered program

### Transport measurement and failure taxonomy

The first slice measures target-list HTTP, target selection, WebSocket connect,
and method/event waits with typed internal diagnostics. It establishes a
public-safe failure-stage vocabulary and a bounded live probe. Timing remains
probe/internal evidence initially; common success-envelope metadata is not
added implicitly. The promoted measurement plan adds a narrowly typed
`failure_stage` field to transport error details so failures encountered during
ordinary operation can be classified without exposing raw transport values.

The measurement must include stale-target detectability. In a dedicated
read-only diagnostic path, a failed WebSocket connection may be followed by one
bounded re-discovery solely to classify whether the selected endpoint was
unchanged, changed for the same selection, disappeared, became ambiguous, or
could not be diagnosed. That diagnostic re-discovery is not retry behavior and
does not make the original command successful.

Live evidence reports only bounded-run counts and latency summaries. It does
not claim a repository-wide failure rate. Deterministic fixtures, rather than
variable live observations, prove classification and deadline behavior.

### Pre-dispatch transport resilience

After measurement evidence is reviewed, a separate ExecPlan may add bounded
retry only before a CDP method can have been dispatched: target listing, target
selection, WebSocket dialing, and one stale-endpoint refresh. The plan must use
one absolute budget and preserve explicit target selection. It must not re-run
an operation sequence after a method call fails.

Retry policy defaults, timeout overrides, backoff shape, and public diagnostic
fields are not fixed by this strategy. They are decided from measured evidence
and deterministic fault-injection tests. If no material transient failure is
observed or reproducible, this slice may close as no-go.

### Connection and evaluation topology audit

A read-only audit determines which workflows actually perform redundant target
discovery, WebSocket connection, or pure-read Runtime evaluations within one
invocation. Existing `connect_runtime()` paths normally connect once, while
multi-target workflows may reconnect intentionally. No optimization is
promised before the audit identifies a removable round trip and proves that
removal preserves target ownership, error attribution, deadlines, and
post-check behavior.

### Recovery-semantics inventory

Recovery is a property of a particular failure after a particular execution
stage, not a static `idempotent: bool` attached to a command name. A later
inventory should classify real failure paths and then decide whether a public
closed vocabulary is useful. Provisional candidate meanings include retrying
the same request, re-discovering before retry, waiting before retry, refusing
automatic retry, and requiring user action. These are design prompts, not a
frozen JSON contract.

Screenshot file writes, chart workflows that mutate and restore state, process
launch, and mutations with uncertain post-dispatch outcomes demonstrate why a
single command-level boolean is insufficient.

### Bounded wait and input preconditions

Named, read-only wait conditions and command-specific focus/visibility checks
may improve observe-act-verify workflows. They need separate source and
contract designs. A future wait surface must accept a closed set of conditions,
not arbitrary JavaScript, and retain absolute timeout and poll-interval bounds.
Input preconditions must be operation-specific because some keyboard or pointer
commands intentionally target the currently active surface rather than a
single discoverable element.

### Shared-connection feasibility

Only measurements after pre-dispatch hardening and topology cleanup can justify
connection sharing. A feasibility gate may compare a foreground bounded
stdin/JSONL session, an explicitly started bounded-lifetime local broker, and
dropping both.

A broker is a background service even when it has idle and absolute TTLs. Its
feasibility therefore requires an explicit policy decision, local IPC security,
single-instance and version-skew behavior, Windows named-pipe investigation,
per-target serialization, and visible transport ownership. If explored,
connection mode must be explicit, such as direct, broker-required, or
broker-auto, and results must identify the selected mode. Silent fallback is
not an acceptable final contract.

The gate must consider more than connection latency. Evidence should include
the observed stale-endpoint classifications, the share recoverable through
ordinary re-discovery, the share that event subscription could detect before a
command fails, and the operational cost of another process. If measurement
shows little value, both shared-connection candidates are dropped.

Renderer lifecycle is a separate measurement axis from connection persistence.
Future feasibility work should compare active-target ownership, app-tab versus
CDP-target identity, `Page.bringToFront` or equivalent visibility transitions
and their side effects, and whether a nonvisual result event can expose
materialization before a command fails. A persistent WebSocket alone is not
evidence that hidden renderer content becomes ready, and indicator-search
readiness by itself does not justify promoting a broker.

## Promotion rules

The v0.29 roadmap may sequence measurement, pre-dispatch resilience, topology
audit, and recovery-semantics inventory, but only measurement starts as an
active ExecPlan. Each later item receives a fresh plan after its prerequisite
evidence is reviewed. Public envelope changes, method-post-dispatch restart,
bounded wait commands, input-precondition expansion, session protocols, and
broker implementation are not implied by roadmap placement.

Documentation and runtime skills are updated in the same slice as any
user-visible contract. They are not deferred to one final documentation phase.

## Success criteria for the program

The program is useful if it produces reproducible answers to these questions:

- Where does a Desktop-backed command spend time and where does it fail?
- Which transient failures can be absorbed before method dispatch under one
  absolute budget?
- Which repeated transport or evaluation work is actually redundant?
- What recovery action, if any, is safe for a specific observed failure?
- Does measured evidence justify a shared connection or a new wait surface?

Live failure-rate or latency improvement is not an unconditional acceptance
requirement. A zero-failure baseline or normal Desktop variance can make such a
claim impossible. Deterministic injected failures prove behavior; bounded live
runs provide non-regression and operational evidence.

## Revision history

- 2026-07-17: converted the original multi-phase ExecPlan into this strategy
  note after review found that it combined several independently shippable
  contracts. Separated pre-dispatch retry from post-dispatch operation restart,
  replaced command-level idempotency with provisional failure-specific
  recovery semantics, made broker adoption an explicit policy gate, removed
  silent fallback, and added stale-target detectability requirements. The
  transport measurement slice now has its own ExecPlan.
