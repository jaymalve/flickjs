import { useRef, useCallback } from "react";
import { fx, useFx } from "@flickjs/react";

// ── Signals — state lives outside components ──
const messages = fx<string[]>([]);
const input = fx("");
const status = fx("idle");

// ── Shared mutable render counts (refs, not state — no re-render loops) ──
const counts = { Header: 0, Messages: 0, Typing: 0, Input: 0 };

function Badge({ n }: { n: number }) {
  return (
    <span
      style={{
        fontFamily: "ui-monospace, SFMono-Regular, Menlo, Consolas, monospace",
        fontSize: 11,
        color: n === 0 ? "#4ade80" : n <= 5 ? "#fbbf24" : "#f87171",
        opacity: 0.9,
      }}
    >
      {n}
    </span>
  );
}

function Header() {
  const s = useFx(status);
  const c = useRef(0);
  c.current++;
  counts.Header = c.current;

  return (
    <div style={card}>
      <Row label="Header" count={c.current} />
      <span style={{ fontSize: 12, color: "#a6a6b8" }}>
        {s === "streaming" ? "● AI Connected" : "○ Idle"}
      </span>
    </div>
  );
}

function MessageList() {
  const msgs = useFx(messages);
  const c = useRef(0);
  c.current++;
  counts.Messages = c.current;

  return (
    <div style={card}>
      <Row label="MessageList" count={c.current} />
      <div style={{ maxHeight: 100, overflowY: "auto", fontSize: 12, color: "#a6a6b8" }}>
        {msgs.length === 0 ? (
          <span style={{ fontStyle: "italic", opacity: 0.5 }}>No messages yet</span>
        ) : (
          msgs.map((m, i) => (
            <div key={i} style={{ padding: "3px 0", borderBottom: "1px solid hsl(240 4% 16%)" }}>
              {m}
            </div>
          ))
        )}
      </div>
    </div>
  );
}

function TypingIndicator() {
  const s = useFx(status);
  const c = useRef(0);
  c.current++;
  counts.Typing = c.current;

  return (
    <div style={card}>
      <Row label="TypingIndicator" count={c.current} />
      {s === "streaming" && (
        <span style={{ fontSize: 12, color: "#4ade80" }}>AI is typing...</span>
      )}
    </div>
  );
}

function InputBar() {
  const value = useFx(input);
  const c = useRef(0);
  c.current++;
  counts.Input = c.current;

  const handleSubmit = useCallback(() => {
    const v = input();
    if (!v.trim()) return;
    messages.set((prev) => [...prev, `You: ${v}`]);
    input.set("");
  }, []);

  return (
    <div style={card}>
      <Row label="InputBar" count={c.current} />
      <div style={{ display: "flex", gap: 8 }}>
        <input
          style={inputStyle}
          value={value}
          onChange={(e) => input.set(e.target.value)}
          placeholder="Type here..."
        />
        <button style={btnSecondary} onClick={handleSubmit}>
          Send
        </button>
      </div>
    </div>
  );
}

function Total() {
  // Subscribe to all signals so this component re-renders when any of them change,
  // which lets us read the updated mutable counts object
  useFx(messages);
  useFx(input);
  useFx(status);

  const total = counts.Header + counts.Messages + counts.Typing + counts.Input;
  return (
    <div style={{ ...card, marginTop: 4 }}>
      <div style={{ display: "flex", justifyContent: "space-between", fontSize: 13, fontWeight: 600, color: "#fafafa" }}>
        <span>Total re-renders</span>
        <span
          style={{
            fontFamily: "ui-monospace, SFMono-Regular, Menlo, Consolas, monospace",
            color: total <= 10 ? "#4ade80" : total <= 30 ? "#fbbf24" : "#f87171",
          }}
        >
          {total}
        </span>
      </div>
    </div>
  );
}

export function FxSide() {
  const streamRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const startStreaming = useCallback(() => {
    if (streamRef.current) return;
    const tokens = "This is a simulated AI response streaming token by token to show how fx signals only re-render the exact subscriber".split(" ");
    let i = 0;

    status.set("streaming");
    messages.set((prev) => [...prev, "AI: "]);

    streamRef.current = setInterval(() => {
      if (i >= tokens.length) {
        clearInterval(streamRef.current!);
        streamRef.current = null;
        status.set("idle");
        return;
      }
      messages.set((prev) => {
        const updated = [...prev];
        updated[updated.length - 1] += (i > 0 ? " " : "") + tokens[i];
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
    messages.set([]);
    input.set("");
    status.set("idle");
    counts.Header = 0;
    counts.Messages = 0;
    counts.Typing = 0;
    counts.Input = 0;
  }, []);

  return (
    <div>
      <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 12 }}>
        <span style={{ fontSize: 16, fontWeight: 600, color: "#fafafa" }}>fx</span>
        <span style={tagGreen}>@flickjs/react</span>
      </div>

      <div style={{ display: "flex", gap: 8, marginBottom: 12 }}>
        <button style={btnPrimary} onClick={startStreaming}>
          Stream AI
        </button>
        <button style={btnSecondary} onClick={reset}>
          Reset
        </button>
      </div>

      <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
        <Header />
        <MessageList />
        <TypingIndicator />
        <InputBar />
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
        display: "flex",
        justifyContent: "space-between",
        alignItems: "center",
        marginBottom: 6,
        fontSize: 12,
        fontWeight: 600,
        color: "#a6a6b8",
      }}
    >
      <span>{label}</span>
      <Badge n={count} />
    </div>
  );
}

// ── Styles matching FlickJS landing page ──

const card: React.CSSProperties = {
  background: "hsl(240 6% 3%)",
  borderRadius: 4,
  padding: "10px 14px",
  boxShadow:
    "inset 0 1px 0 0 hsl(0 0% 100% / 0.03), 0 0 0 1px hsl(0 0% 100% / 0.06)",
};

const inputStyle: React.CSSProperties = {
  flex: 1,
  padding: "6px 10px",
  fontSize: 13,
  fontFamily: "inherit",
  background: "#000",
  border: "1px solid hsl(240 4% 16%)",
  borderRadius: 4,
  color: "#fafafa",
  outline: "none",
};

const btnPrimary: React.CSSProperties = {
  flex: 1,
  padding: "7px 14px",
  fontSize: 13,
  fontWeight: 600,
  fontFamily: "inherit",
  color: "#000",
  background: "#fff",
  border: "none",
  borderRadius: 4,
  cursor: "pointer",
};

const btnSecondary: React.CSSProperties = {
  padding: "7px 14px",
  fontSize: 13,
  fontWeight: 600,
  fontFamily: "inherit",
  color: "#fafafa",
  background: "transparent",
  border: "1px solid hsl(0 0% 100% / 0.15)",
  borderRadius: 4,
  cursor: "pointer",
};

const tagGreen: React.CSSProperties = {
  fontSize: 10,
  fontWeight: 600,
  color: "#4ade80",
  background: "hsl(140 40% 8%)",
  padding: "2px 8px",
  borderRadius: 9999,
  textTransform: "uppercase",
  letterSpacing: 0.5,
};
