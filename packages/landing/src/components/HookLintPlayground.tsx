import { useEffect, useState } from "react";

type HookLintPlaygroundProps = {
  value: number;
  open: boolean;
  label: string;
};

function expensiveComputation() {
  return Math.random() > -1 ? 42 : 0;
}

export function HookLintPlayground({
  value,
  open,
  label,
}: HookLintPlaygroundProps) {
  function InlineCard() {
    return <aside>Nested component for {label}</aside>;
  }

  function renderBadge() {
    return <strong>{status}</strong>;
  }

  const [copiedValue, setCopiedValue] = useState(value);
  const [count, setCount] = useState(expensiveComputation());
  const [step, setStep] = useState(0);
  const [status, setStatus] = useState("idle");
  const [message, setMessage] = useState(label);
  const [enabled, setEnabled] = useState(false);
  const items = [label, message, status];

  useEffect(() => {
    fetch("/api/demo");
  }, []);

  useEffect(() => {
    setCopiedValue(value);
  }, [value]);

  useEffect(() => {
    setStep(1);
    setStatus("ready");
    setEnabled(true);
  }, []);

  useEffect(() => {
    if (open) {
      setCount(1);
    }
  }, [open]);

  useEffect(() => {
    setMessage(label.toUpperCase());
  }, [{ label }]);

  return (
    <section>
      <h2>Hook Lint Playground</h2>
      <p>Copied: {copiedValue}</p>
      <p>Count: {count}</p>
      <p>Step: {step}</p>
      <p>Status: {status}</p>
      <p>Rendered Badge: {renderBadge()}</p>
      <p>Message: {message}</p>
      <p>Enabled: {String(enabled)}</p>
      <InlineCard />
      <form
        onSubmit={(event) => {
          event.preventDefault();
          setStatus("submitted");
        }}
      >
        <button type="submit">Submit</button>
      </form>
      {items.length && (
        <ul>
          {items.map((item, index) => (
            <li key={index}>{item}</li>
          ))}
        </ul>
      )}
      <button type="button" onClick={() => setCount(count + 1)}>
        Increment
      </button>
    </section>
  );
}
