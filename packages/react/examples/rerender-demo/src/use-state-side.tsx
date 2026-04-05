import { useState, useRef, useCallback } from 'react';

// ── Shared mutable render counts (refs, not state — no re-render loops) ──
const counts = { Header: 0, Messages: 0, Typing: 0, Input: 0 };

function Badge({ n }: { n: number }) {
  return (
    <span
      style={{
        fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Consolas, monospace',
        fontSize: 11,
        color: n === 0 ? '#4ade80' : n <= 5 ? '#fbbf24' : '#f87171',
        opacity: 0.9
      }}
    >
      {n}
    </span>
  );
}

function Header() {
  const c = useRef(0);
  c.current++;
  counts.Header = c.current;

  return (
    <div style={card}>
      <Row label="Header" count={c.current} />
      {/* <span style={{ fontSize: 12, color: '#a6a6b8' }}>
        {status === 'streaming' ? '● AI Connected' : '○ Idle'}
      </span> */}
    </div>
  );
}

function MessageList({ messages }: { messages: string[] }) {
  const c = useRef(0);
  c.current++;
  counts.Messages = c.current;

  return (
    <div style={card}>
      <Row label="MessageList" count={c.current} />
      <div style={{ maxHeight: 100, overflowY: 'auto', fontSize: 12, color: '#a6a6b8' }}>
        {messages.length === 0 ? (
          <span style={{ fontStyle: 'italic', opacity: 0.5 }}>No messages yet</span>
        ) : (
          messages.map((m, i) => (
            <div key={i} style={{ padding: '3px 0', borderBottom: '1px solid hsl(240 4% 16%)' }}>
              {m}
            </div>
          ))
        )}
      </div>
    </div>
  );
}

function TypingIndicator({ status }: { status: string }) {
  const c = useRef(0);
  c.current++;
  counts.Typing = c.current;

  return (
    <div style={card}>
      <Row label="TypingIndicator" count={c.current} />
      {status === 'streaming' && (
        <span style={{ fontSize: 12, color: '#4ade80' }}>AI is typing...</span>
      )}
    </div>
  );
}

function InputBar({
  input,
  onChange,
  onSubmit
}: {
  input: string;
  onChange: (v: string) => void;
  onSubmit: () => void;
}) {
  const c = useRef(0);
  c.current++;
  counts.Input = c.current;

  return (
    <div style={card}>
      <Row label="InputBar" count={c.current} />
      <div style={{ display: 'flex', gap: 8 }}>
        <input
          style={inputStyle}
          value={input}
          onChange={(e) => onChange(e.target.value)}
          placeholder="Type here..."
        />
        <button style={btnSecondary} onClick={onSubmit}>
          Send
        </button>
      </div>
    </div>
  );
}

function Total() {
  const total = counts.Header + counts.Messages + counts.Typing + counts.Input;
  return (
    <div style={{ ...card, marginTop: 4 }}>
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          fontSize: 13,
          fontWeight: 600,
          color: '#fafafa'
        }}
      >
        <span>Total re-renders</span>
        <span
          style={{
            fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Consolas, monospace',
            color: total <= 10 ? '#4ade80' : total <= 30 ? '#fbbf24' : '#f87171'
          }}
        >
          {total}
        </span>
      </div>
    </div>
  );
}

export function UseStateSide() {
  const [messages, setMessages] = useState<string[]>([]);
  const [input, setInput] = useState('');
  const [status, setStatus] = useState('idle');
  const streamRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const handleSubmit = useCallback(() => {
    if (!input.trim()) return;
    setMessages((prev) => [...prev, `You: ${input}`]);
    setInput('');
  }, [input]);

  const startStreaming = useCallback(() => {
    if (streamRef.current) return;
    const tokens =
      'This is a simulated AI response streaming token by token to show how re-renders cascade through every component with useState'.split(
        ' '
      );
    let i = 0;

    setStatus('streaming');
    setMessages((prev) => [...prev, 'AI: ']);

    streamRef.current = setInterval(() => {
      if (i >= tokens.length) {
        clearInterval(streamRef.current!);
        streamRef.current = null;
        setStatus('idle');
        return;
      }
      setMessages((prev) => {
        const updated = [...prev];
        updated[updated.length - 1] += (i > 0 ? ' ' : '') + tokens[i];
        return updated;
      });
      i++;
    }, 60);
  }, []);

  const reset = useCallback(() => {
    if (streamRef.current) {
      clearInterval(streamRef.current);
      streamRef.current = null;
    }
    setMessages([]);
    setInput('');
    setStatus('idle');
    counts.Header = 0;
    counts.Messages = 0;
    counts.Typing = 0;
    counts.Input = 0;
  }, []);

  return (
    <div>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 12 }}>
        <span style={{ fontSize: 16, fontWeight: 600, color: '#fafafa' }}>useState</span>
        {/* <span style={tag}>React default</span> */}
      </div>

      <div style={{ display: 'flex', gap: 8, marginBottom: 12 }}>
        <button style={btnPrimary} onClick={startStreaming}>
          Stream
        </button>
        <button style={btnSecondary} onClick={reset}>
          Reset
        </button>
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
        <Header />
        <MessageList messages={messages} />
        <TypingIndicator status={status} />
        <InputBar input={input} onChange={setInput} onSubmit={handleSubmit} />
        <Total />
      </div>
    </div>
  );
}

// ── Shared sub-components ──

function Row({ label, count }: { label: string; count: number }) {
  return (
    <div
      style={{
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: 6,
        fontSize: 12,
        fontWeight: 600,
        color: '#a6a6b8'
      }}
    >
      <span>{label}</span>
      <Badge n={count} />
    </div>
  );
}

// ── Styles matching FlickJS landing page ──

const card: React.CSSProperties = {
  background: 'hsl(240 6% 3%)',
  borderRadius: 4,
  padding: '10px 14px',
  boxShadow: 'inset 0 1px 0 0 hsl(0 0% 100% / 0.03), 0 0 0 1px hsl(0 0% 100% / 0.06)'
};

const inputStyle: React.CSSProperties = {
  flex: 1,
  padding: '6px 10px',
  fontSize: 13,
  fontFamily: 'inherit',
  background: '#000',
  border: '1px solid hsl(240 4% 16%)',
  borderRadius: 4,
  color: '#fafafa',
  outline: 'none'
};

const btnPrimary: React.CSSProperties = {
  flex: 1,
  padding: '7px 14px',
  fontSize: 13,
  fontWeight: 600,
  fontFamily: 'inherit',
  color: '#000',
  background: '#fff',
  border: 'none',
  borderRadius: 4,
  cursor: 'pointer'
};

const btnSecondary: React.CSSProperties = {
  padding: '7px 14px',
  fontSize: 13,
  fontWeight: 600,
  fontFamily: 'inherit',
  color: '#fafafa',
  background: 'transparent',
  border: '1px solid hsl(0 0% 100% / 0.15)',
  borderRadius: 4,
  cursor: 'pointer'
};

const tag: React.CSSProperties = {
  fontSize: 10,
  fontWeight: 600,
  color: '#a6a6b8',
  background: 'hsl(240 4% 10%)',
  padding: '2px 8px',
  borderRadius: 9999,
  textTransform: 'uppercase',
  letterSpacing: 0.5
};
