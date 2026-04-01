# Surgical Re-renders: How `fx` signals outperform React state

## How React state updates work (the problem)

Take a typical chat app:

```tsx
function ChatApp() {
  const [messages, setMessages] = useState([])
  const [input, setInput] = useState('')
  const [status, setStatus] = useState('idle')

  return (
    <div>
      <Header status={status} />
      <MessageList messages={messages} />
      <TypingIndicator status={status} />
      <InputBar input={input} onChange={setInput} onSubmit={handleSubmit} />
    </div>
  )
}
```

When the user types a single character, `setInput('h')` fires. Here's what React does internally:

```
1. setInput('h') called
2. React marks ChatApp as "needs re-render"
3. React re-executes the ENTIRE ChatApp function
4. ChatApp returns a new virtual DOM tree (all 4 children)
5. React diffs the old vDOM tree against the new one
   - Header: props unchanged? skip (but still compared)
   - MessageList: props unchanged? skip (but still compared)
   - TypingIndicator: props unchanged? skip (but still compared)
   - InputBar: props changed (input="h") → update DOM
6. React commits the diff to the real DOM
```

**The problem**: even though only `input` changed, React re-ran the entire `ChatApp` function, recreated all JSX, and diffed all 4 children. With 50 messages in `MessageList`, that's 50 message objects being compared in the diff — every single keystroke.

You can optimize with `React.memo`:

```tsx
const MessageList = React.memo(({ messages }) => {
  return messages.map(m => <div key={m.id}>{m.content}</div>)
})
```

But `React.memo` is **opt-in per component**, does a shallow comparison of all props every render, and the parent still re-executes. It reduces DOM work, but not JS work.

## How fx + useFx works (surgical re-renders)

```tsx
import { fx, useFx } from '@flickjs/react'

// State lives OUTSIDE components — just plain JS
const messages = fx([])
const input = fx('')
const status = fx('idle')

function ChatApp() {
  return (
    <div>
      <Header />
      <MessageList />
      <TypingIndicator />
      <InputBar />
    </div>
  )
}

function Header() {
  const s = useFx(status)    // subscribes to status only
  return <div>{s}</div>
}

function MessageList() {
  const msgs = useFx(messages)  // subscribes to messages only
  return msgs.map(m => <div key={m.id}>{m.content}</div>)
}

function TypingIndicator() {
  const s = useFx(status)    // subscribes to status only
  return s === 'streaming' ? <div>AI is typing...</div> : null
}

function InputBar() {
  const value = useFx(input)  // subscribes to input only
  return <input value={value} onChange={e => input.set(e.target.value)} />
}
```

Now when the user types `'h'`, here's what happens:

```
1. input.set('h') called
2. fx updates the value and fires subscribers
3. The ONLY subscriber to `input` is InputBar's useFx hook
4. useSyncExternalStore calls getSnapshot() → 'h' (changed)
5. React re-renders ONLY InputBar
6. ChatApp: NOT re-rendered
7. Header: NOT re-rendered
8. MessageList: NOT re-rendered (50 messages never touched)
9. TypingIndicator: NOT re-rendered
```

**No parent re-execution. No prop diffing. No memo needed.** React only touches the one component that actually reads the changed signal.

## What happens during AI streaming

This is where the difference compounds. During streaming, the AI sends ~50 tokens/second. Each token updates the messages array:

**With React useState:**

```
Token arrives → setMessages([...prev, updatedMsg])
  → ChatApp re-renders (every 20ms)
    → Header re-renders (props diff)
    → MessageList re-renders (50 messages diffed)
    → TypingIndicator re-renders (props diff)
    → InputBar re-renders (props diff)
```

50 times per second, the entire tree re-executes and diffs. Even with `React.memo`, every component's props are shallow-compared 50 times per second.

**With fx + useFx:**

```
Token arrives → messages.set([...prev, updatedMsg])
  → ONLY MessageList re-renders (it's the only subscriber)
  → Header: untouched
  → TypingIndicator: untouched
  → InputBar: untouched
  → ChatApp: untouched
```

50 times per second, only `MessageList` re-renders. Everything else is completely inert.

## The mechanism under the hood

Here's exactly what `useFx` does internally with `useSyncExternalStore`:

```
                    ┌─────────────────────────────────────────┐
                    │          fx signal (e.g. input)          │
                    │                                         │
                    │  value: 'h'                             │
                    │  subs: Set { executeA }                 │
                    │         ▲                               │
                    └─────────│───────────────────────────────┘
                              │
            ┌─────────────────┘ (auto-tracked by run())
            │
   ┌────────│────────────────────────────────────────────┐
   │  useSyncExternalStore inside InputBar               │
   │                                                     │
   │  subscribe = (onStoreChange) => {                   │
   │    dispose = run(() => {   ◄── creates effect       │
   │      signal()              ◄── reads fx, gets       │
   │                                tracked in subs      │
   │      onStoreChange()       ◄── tells React to       │
   │    })                          check snapshot        │
   │    return dispose                                    │
   │  }                                                   │
   │                                                     │
   │  getSnapshot = () => signal()  → returns 'h'        │
   │                                                     │
   │  React compares: Object.is('', 'h') → false         │
   │  → schedules re-render of THIS component only       │
   └─────────────────────────────────────────────────────┘
```

When `input.set('h')` fires:

1. `fx` updates its internal value from `''` to `'h'`
2. `fx` iterates its `subs` Set — only `executeA` (InputBar's run callback)
3. `executeA` fires → calls `signal()` (re-tracks) → calls `onStoreChange()`
4. React calls `getSnapshot()` → gets `'h'`
5. React compares with previous snapshot `''` → different → re-render InputBar
6. No other component is in this signal's `subs` → nothing else happens

## The key insight

With `useState`, **state is owned by the component**. Updating state re-renders that component and everything below it. You work top-down and optimize with `memo` to stop the cascade.

With `fx`, **state is owned by the signal**. Components are just subscribers. Only the exact subscribers of a changed signal re-render. There's no cascade to stop because there's no cascade in the first place.

```
useState world:               fx world:

    ChatApp ← re-renders      ChatApp ← untouched
    ├─ Header ← re-renders    ├─ Header ← untouched
    ├─ MessageList ← re-renders    ├─ MessageList ← untouched
    ├─ Typing ← re-renders    ├─ Typing ← untouched
    └─ InputBar ← re-renders  └─ InputBar ← RE-RENDERS (only subscriber)
```

That's surgical re-renders — not surgical DOM updates (React still diffs), but **the diff is scoped to only the component that matters**.
